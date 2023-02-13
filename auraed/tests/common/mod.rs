use auraed::AuraedRuntime;
use backoff::backoff::Backoff;
use client::{AuraeConfig, AuthConfig, Client, ClientError, SystemConfig};
use once_cell::sync::Lazy;
use std::future::Future;
use std::time::Duration;
use tokio::sync::OnceCell;

#[macro_export]
macro_rules! retry {
    ($function:expr) => {{
        // TODO: Define a default retry strategy somewhere
        let retry_strategy = ::backoff::ExponentialBackoffBuilder::new()
            .with_initial_interval(::std::time::Duration::from_millis(50)) // 1st retry in 50ms
            .with_multiplier(10.0) // 10x the delay after 1st retry (500ms)
            .with_randomization_factor(0.5) // with a randomness of +/-50% (250-750ms)
            .with_max_interval(::std::time::Duration::from_secs(3)) // but never delay more than 3s
            .with_max_elapsed_time(Some(::std::time::Duration::from_secs(20))) // or 20s total
            .build();

        ::backoff::future::retry(
            retry_strategy,
            || async {
                match $function {
                    Ok(res) => Ok(res),
                    Err(e) if e.code() == ::tonic::Code::Unknown && e.message() == "transport error" => {
                        Err(e)?;
                        unreachable!();
                    }
                    Err(e) => Err(::backoff::Error::Permanent(e))
                }
            },
        )
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
    let runtime = AuraedRuntime::default();
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
        system: SystemConfig { socket: socket.clone() },
    };

    let _ = tokio::spawn(async move {
        auraed::run(runtime, Some(socket), false, false).await.unwrap()
    });

    let mut retry_strategy = backoff::ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_millis(50)) // 1st retry in 50ms
        .with_multiplier(10.0) // 10x the delay after 1st retry (500ms)
        .with_randomization_factor(0.5) // with a randomness of +/-50% (250-750ms)
        .with_max_interval(Duration::from_secs(3)) // but never delay more than 3s
        .with_max_elapsed_time(Some(Duration::from_secs(20))) // or 20s total
        .build();

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
