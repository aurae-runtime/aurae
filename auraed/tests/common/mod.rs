/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */
use auraed::{AuraedPath, AuraedRuntime};
use backoff::{
    backoff::Backoff, exponential::ExponentialBackoff,
    ExponentialBackoffBuilder, SystemClock,
};
use client::{
    AuraeConfig, AuraeSocket, AuthConfig, Client, ClientError, SystemConfig,
};
use once_cell::sync::Lazy;
use std::{future::Future, time::Duration};
use tokio::sync::OnceCell;

pub mod cells;
pub mod observe;

#[macro_export]
macro_rules! retry {
    ($function:expr) => {{
        let retry_strategy = $crate::common::default_retry_strategy();

        ::backoff::future::retry(retry_strategy, || async {
            match $function {
                Ok(res) => Ok(res),
                Err(e)
                    if e.code() == ::tonic::Code::Unknown
                        && e.message() == "transport error" =>
                {
                    Err(e)?;
                    unreachable!();
                }
                Err(e) => Err(::backoff::Error::Permanent(e)),
            }
        })
        .await
    }};
}

static RT: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());

pub fn test<F>(f: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let runtime = Lazy::force(&RT);
    let handle = runtime.handle();
    let _eg = handle.enter();
    futures::executor::block_on(f)
}

async fn run_auraed() -> Client {
    let socket = std::env::temp_dir()
        .join(format!("{}.socket", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .to_string();

    // TODO: using "~/.aurae/pki/ca.crt" errors with file not found (confirmed it exists)
    //   even though that is the default in default.config.toml in auraescript.
    let client_config = AuraeConfig {
        auth: AuthConfig {
            ca_crt: "/etc/aurae/pki/ca.crt".to_string(),
            client_crt: "/etc/aurae/pki/_signed.client.nova.crt".to_string(),
            client_key: "/etc/aurae/pki/client.nova.key".to_string(),
        },
        system: SystemConfig {
            socket: AuraeSocket::Path(socket.clone().into()),
        },
    };

    let _ = tokio::spawn(async move {
        let mut runtime = AuraedRuntime::default();
        runtime.auraed = AuraedPath::from_path("auraed");

        auraed::run(runtime, Some(socket), false, false).await.unwrap()
    });

    let mut retry_strategy = default_retry_strategy();

    let client = loop {
        match Client::new(client_config.clone()).await {
            Ok(client) => break Ok(client),
            e @ Err(ClientError::ConnectionError(_)) => {
                if let Some(delay) = retry_strategy.next_backoff() {
                    tokio::time::sleep(delay).await
                } else {
                    break e;
                }
            }
            e => break e,
        }
    }
    .expect("failed to create client");

    client
}

static CLIENT: OnceCell<Client> = OnceCell::const_new();

pub async fn auraed_client() -> Client {
    async fn inner() -> Client {
        run_auraed().await
    }

    CLIENT.get_or_init(inner).await.clone()
}

// pub async fn remote_auraed_client(ip: String, scope_id: u32) -> Client {
//     let client_config = AuraeConfig {
//         auth: AuthConfig {
//             ca_crt: "/etc/aurae/pki/ca.crt".to_string(),
//             client_crt: "/etc/aurae/pki/_signed.client.nova.crt".to_string(),
//             client_key: "/etc/aurae/pki/client.nova.key".to_string(),
//         },
//         system: SystemConfig { socket: AuraeSocket::IPv6 { ip, scope_id } },
//     };
//     let client = Client::new(client_config.clone())
//         .await
//         .expect("failed to create client");
//
//     client
// }

pub fn default_retry_strategy() -> ExponentialBackoff<SystemClock> {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_millis(50)) // 1st retry in 50ms
        .with_multiplier(10.0) // 10x the delay after 1st retry (500ms)
        .with_randomization_factor(0.5) // with a randomness of +/-50% (250-750ms)
        .with_max_interval(Duration::from_secs(3)) // but never delay more than 3s
        .with_max_elapsed_time(Some(Duration::from_secs(20))) // or 20s total
        .build()
}