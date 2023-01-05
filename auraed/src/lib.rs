/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

//! Systems daemon built for higher order simple, safe, secure multi-tenant
//! distributed systems.
//!
//! Runs as pid 1 (init) and serves standard library functionality over a mTLS
//! backed gRPC server.
//!
//! The Aurae Daemon (auraed) is the main server implementation of the Aurae
//! Standard Library.
//!
//! See [`The Aurae Standard Library`] for API reference.
//!
//! [`The Aurae Standard Library`]: https://aurae.io/stdlib
// Lint groups: https://doc.rust-lang.org/rustc/lints/groups.html
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    unconditional_recursion,
    unused_comparisons,
    while_true
)]
#![warn(missing_debug_implementations,
    // TODO: missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]
#![warn(clippy::unwrap_used)]
#![warn(missing_docs)]
#![allow(dead_code)]

use anyhow::Context;
use aurae_proto::runtime::cell_service_server::CellServiceServer;
use aurae_proto::runtime::pod_service_server::PodServiceServer;
use clap::Parser;
use runtime::CellService;
use runtime::PodService;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tracing::{error, info, trace};

pub mod init;
pub mod logging;
mod observe;
mod runtime;
mod schedule;
mod signal_handlers;

/// Default Unix domain socket path for `auraed`.
///
/// Warning: This socket is created (by default) with user
/// mode 0o766 which allows for unprivileged access to the
/// auraed daemon which can in turn be used to execute privileged
/// processes and commands. Access to the socket must be governed
/// by an appropriate mTLS Authorization setting in order to maintain
/// a secure multi tenant system.
const AURAE_SOCK: &str = "/var/run/aurae/aurae.sock";
const EXIT_OKAY: i32 = 0;
const EXIT_ERROR: i32 = 1;

/// Command line options for auraed.
///
/// Defines the configurable options which can be used to populate
/// an AuraeRuntime structure.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct AuraedOptions {
    /// The signed server certificate. Defaults to /etc/aurae/pki/_signed.server.crt
    #[clap(
        long,
        value_parser,
        default_value = "/etc/aurae/pki/_signed.server.crt"
    )]
    server_crt: String,
    /// The secret server key. Defaults to /etc/aurae/pki/server.key
    #[clap(long, value_parser, default_value = "/etc/aurae/pki/server.key")]
    server_key: String,
    /// The CA certificate. Defaults to /etc/aurae/pki/ca.crt
    #[clap(long, value_parser, default_value = "/etc/aurae/pki/ca.crt")]
    ca_crt: String,
    /// Aurae socket path. Defaults to /var/run/aurae/aurae.sock
    #[clap(short, long, value_parser, default_value = AURAE_SOCK)]
    socket: String,
    /// Toggle verbosity. Default false
    #[clap(short, long, alias = "ritz")]
    verbose: bool,
    /// Run auraed as a nested instance of itself in an Aurae cell.
    #[clap(long)]
    nested: bool,
}

#[allow(missing_docs)] // TODO
pub async fn daemon() -> i32 {
    let options = AuraedOptions::parse();

    // Initializes Logging and prepares system if auraed is run as pid=1
    init::init(options.verbose, options.nested).await;

    info!("Starting Aurae Daemon Runtime");
    info!("Options: {options:#?}");
    info!("Aurae Daemon is pid {}", std::process::id());

    let runtime = AuraedRuntime {
        server_crt: PathBuf::from(options.server_crt),
        server_key: PathBuf::from(options.server_key),
        ca_crt: PathBuf::from(options.ca_crt),
        socket: PathBuf::from(options.socket),
    };

    let e = runtime.run().await;
    if e.is_err() {
        error!("{:?}", e);
    }

    if e.is_err() {
        EXIT_ERROR
    } else {
        EXIT_OKAY
    }
}

/// Each instance of Aurae holds internal state in memory. Below are the
/// settings which can be configured for a given Aurae daemon instance.
///
/// Note: These fields represent file paths and not the actual authentication
/// material. Each new instance of a subsystem will read these from the local
/// filesystem at runtime in order to authenticate.
#[derive(Debug)]
struct AuraedRuntime {
    /// Certificate Authority for an organization or mesh of Aurae instances.
    pub ca_crt: PathBuf,
    /// The signed server X509 certificate for this unique instance.
    pub server_crt: PathBuf,
    /// The secret key for this unique instance.
    pub server_key: PathBuf,
    /// Configurable socket path. Defaults to the value of
    /// `pub const AURAE_SOCK`
    pub socket: PathBuf,
    // /// Provides logging channels to expose auraed logging via grpc
    //pub log_collector: Arc<LogChannel>,
}

/// Primary daemon structure. Holds state and memory for this instance of
/// Aurae.
impl AuraedRuntime {
    /// Starts the runtime loop for the daemon.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = fs::remove_file(&self.socket);
        let sock_path = Path::new(&self.socket)
            .parent()
            .ok_or("unable to find socket path")?;
        // Create socket directory
        tokio::fs::create_dir_all(sock_path).await.with_context(|| {
            format!(
                "Failed to create directory for socket: {}",
                self.socket.display()
            )
        })?;
        trace!("{:#?}", self);

        let server_crt =
            tokio::fs::read(&self.server_crt).await.with_context(|| {
                format!(
                    "Failed to read server certificate: {}",
                    self.server_crt.display()
                )
            })?;
        let server_key = tokio::fs::read(&self.server_key).await?;
        let server_identity = Identity::from_pem(server_crt, server_key);
        info!("Register Server SSL Identity");

        let ca_crt = tokio::fs::read(&self.ca_crt).await?;
        let ca_crt_pem = Certificate::from_pem(ca_crt.clone());

        let tls = ServerTlsConfig::new()
            .identity(server_identity)
            .client_ca_root(ca_crt_pem);

        info!("Validating SSL Identity and Root Certificate Authority (CA)");

        let sock = UnixListener::bind(&self.socket)?;
        let sock_stream = UnixListenerStream::new(sock);
        //let _log_collector = self.log_collector.clone();

        let cell_service = CellService::new();
        let cell_service_server = CellServiceServer::new(cell_service.clone());
        let pod_service = PodService::new();
        let pod_service_server = PodServiceServer::new(pod_service.clone());

        // Run the server concurrently
        // TODO: pass a known-good path to CellService to store any runtime data.
        let server_handle = tokio::spawn(async move {
            Server::builder()
                .tls_config(tls)?
                .add_service(cell_service_server)
                .add_service(pod_service_server)
                // .add_service(ObserveServer::new(ObserveService::new(
                //     log_collector,
                // )))
                .serve_with_incoming_shutdown(sock_stream, async {
                    let _signal = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate(),
                    )
                    .expect("failed to create shutdown signal stream")
                    .recv()
                    .await;

                    info!("Received shutdown signal...");
                })
                .await?;

            info!("gRPC server exited successfully");

            Ok::<_, tonic::transport::Error>(())
        });

        trace!("Setting socket mode {} -> 766", &self.socket.display());

        // We set the mode to 766 for the Unix domain socket.
        // This is what allows non-root users to dial the socket
        // and authenticate with mTLS.
        fs::set_permissions(&self.socket, fs::Permissions::from_mode(0o766))?;
        info!("User Access Socket Created: {}", self.socket.display());

        // Event loop
        let terminate_handle = tokio::spawn(async {
            signal_handlers::terminate(cell_service).await
        });

        let (server_result, _) =
            tokio::try_join!(server_handle, terminate_handle)?;

        if let Err(e) = server_result {
            error!("gRPC server exited with error: {e}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        assert_eq!(AURAE_SOCK, "/var/run/aurae/aurae.sock");
    }
}
