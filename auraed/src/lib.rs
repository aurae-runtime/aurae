/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
//! Whether run as pid 1 (init), or a Container, or a Pod it serves standard library
//! functionality over an mTLS backed gRPC server.
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
#![warn(
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]
#![warn(clippy::unwrap_used)]

use crate::cri::oci::AuraeOCIBuilder;
use crate::cri::runtime_service::RuntimeService;
use crate::ebpf::loader::BpfLoader;
use crate::init::Context as AuraeContext;
use crate::logging::log_channel::LogChannel;
use crate::{
    cells::CellService, discovery::DiscoveryService, init::SocketStream,
    observe::ObserveService, spawn::spawn_auraed_oci_to,
};
use anyhow::Context;
use proto::{
    cells::cell_service_server::CellServiceServer,
    cri::runtime_service_server::RuntimeServiceServer,
    discovery::discovery_service_server::DiscoveryServiceServer,
    observe::observe_service_server::ObserveServiceServer,
};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tonic::transport::server::Connected;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use tracing::{error, info, trace, warn};

mod cells;
mod cri;
mod discovery;
mod ebpf;
mod graceful_shutdown;
pub mod init;
pub mod logging;
mod observe;
mod spawn;

/// Default Unix domain socket path for `auraed`.
///
/// Warning: This socket is created (by default) with user
/// mode 0o766 which allows for unprivileged access to the
/// auraed daemon which can in turn be used to execute privileged
/// processes and commands. Access to the socket must be governed
/// by an appropriate mTLS Authorization setting in order to maintain
/// a secure multi tenant system.
const AURAE_SOCK: &str = "aurae.sock";

/// Default runtime directory for Aurae.
///
/// All aspects of the auraed daemon should respect this value.
///
/// Here is where the auraed daemon will store artifacts such as
/// OCI bundles for containers, the aurae.sock socket file, and
/// runtime pod configuration.
///
/// This is the main "runtime" location for all artifacts that are
/// a consequence of runtime operations.
const AURAE_RUNTIME_DIR: &str = "/var/run/aurae";

/// Default library directory for Aurae.
///
/// All aspects of the auraed library and dependency artifacts
/// should respect this value.
///
/// Here is where the daemon will look for artifacts such as eBPF
/// bytecode (ELF objects/probes) and other dependencies that can
/// optionally be included at runtime.
const AURAE_LIBRARY_DIR: &str = "/var/lib/aurae";

/// Default exit code for successful termination of auraed.
const EXIT_OKAY: i32 = 0;

/// Default exit code for a runtime error of auraed.
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
    /// Aurae socket address.  Depending on context, this should be a file or a network address.
    /// Defaults to ${runtime_dir}/aurae.sock or [::1]:8080 respectively.
    #[clap(short, long, value_parser)]
    socket: Option<String>,
    /// Aurae runtime path.  Defaults to /var/run/aurae.
    #[clap(short, long, value_parser, default_value = AURAE_RUNTIME_DIR)]
    runtime_dir: String,
    /// Aurae library path. Defaults to /var/lib/aurae
    #[clap(short, long, value_parser, default_value = AURAE_LIBRARY_DIR)]
    library: String,
    /// Toggle verbosity. Default false
    #[clap(short, long, alias = "ritz")]
    verbose: bool,
    /// Run auraed as a nested instance of itself in an Aurae cell.
    #[clap(long)]
    nested: bool,
    // Subcommands for the project
    #[clap(subcommand)]
    subcmd: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Spawn {
        #[clap(short, long, value_parser, default_value = ".")]
        output: String,
    },
}

/// This is the core function of the auraed runtime.
pub async fn daemon() -> i32 {
    let options = AuraedOptions::parse();

    match &options.subcmd {
        Some(SubCommands::Spawn { output }) => {
            info!("Spawning Auraed OCI bundle: {}", output);
            spawn_auraed_oci_to(
                output,
                AuraeOCIBuilder::new()
                    .build()
                    .expect("building default oci spec"),
            )
            .expect("spawning");
            return EXIT_OKAY;
        }
        None => {}
    }

    info!("Starting Aurae Daemon Runtime");
    info!("Aurae Daemon is pid {}", std::process::id());

    let runtime = AuraedRuntime {
        server_crt: PathBuf::from(options.server_crt),
        server_key: PathBuf::from(options.server_key),
        ca_crt: PathBuf::from(options.ca_crt),
        runtime_dir: PathBuf::from(options.runtime_dir),
        context: AuraeContext::get(options.nested),
    };

    let e = match init::init(options.verbose, options.nested, options.socket)
        .await
    {
        SocketStream::Tcp(stream) => runtime.run(stream).await,
        SocketStream::Unix(stream) => runtime.run(stream).await,
    };

    if e.is_err() {
        error!("{:?}", e);
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
    /// Configurable runtime directory. Defaults to /var/run/aurae.
    pub runtime_dir: PathBuf,
    /// Context in which this Aurae instance is operating
    pub context: AuraeContext,
    // /// Provides logging channels to expose auraed logging via grpc
    //pub log_collector: Arc<LogChannel>,
}

/// Primary daemon structure. Holds state and memory for this instance of
/// Aurae.
impl AuraedRuntime {
    /// Starts the runtime loop for the daemon.
    pub async fn run<T, IO, IE>(
        &self,
        socket_stream: T,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: tokio_stream::Stream<Item = Result<IO, IE>> + Send + 'static,
        IO: AsyncRead + AsyncWrite + Connected + Unpin + Send + 'static,
        IE: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        trace!("{:#?}", self);

        let server_crt =
            tokio::fs::read(&self.server_crt).await.with_context(|| {
                format!(
                    "Aurae requires a signed TLS certificate to run as a server, but failed to 
                    load: '{}'. Please see https://aurae.io/certs/ for information on best 
                    practices to quickly generate one.",
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
        //let _log_collector = self.log_collector.clone();

        let runtime_dir = Path::new(&self.runtime_dir);
        // Create runtime directory
        tokio::fs::create_dir_all(runtime_dir).await.with_context(|| {
            format!(
                "Failed to create runtime directory: {}",
                self.runtime_dir.display()
            )
        })?;

        // Install eBPF probes in the host Aurae daemon
        let (_bpf_scope, signals) = if self.context == AuraeContext::Cell
            || self.context == AuraeContext::Container
        {
            (None, None)
        } else {
            // TODO: Add flags/options to "opt-out" of the various BPF probes
            info!("Loading eBPF probes");
            let mut bpf_loader = BpfLoader::new();
            let listener = bpf_loader
                .read_and_load_tracepoint_signal_signal_generate()
                .ok();

            if listener.is_none() {
                warn!("Missing eBPF probe. Skipping signal reporting.");
            }

            // Need to move bpf_loader out to prevent it from being dropped
            (Some(bpf_loader), listener)
        };

        // Build gRPC Services
        let (mut health_reporter, health_service) =
            tonic_health::server::health_reporter();

        let cell_service = CellService::new();
        let cell_service_server = CellServiceServer::new(cell_service.clone());
        health_reporter.set_serving::<CellServiceServer<CellService>>().await;

        let discovery_service = DiscoveryService::new();
        let discovery_service_server =
            DiscoveryServiceServer::new(discovery_service);
        health_reporter
            .set_serving::<DiscoveryServiceServer<DiscoveryService>>()
            .await;

        let observe_service = ObserveService::new(
            Arc::new(LogChannel::new(String::from("TODO"))),
            signals,
        );
        let observe_service_server = ObserveServiceServer::new(observe_service);
        health_reporter
            .set_serving::<ObserveServiceServer<ObserveService>>()
            .await;

        // let pod_service = PodService::new(self.runtime_dir.clone());
        // let pod_service_server = PodServiceServer::new(pod_service.clone());
        // health_reporter.set_serving::<PodServiceServer<PodService>>().await;
        let runtime_service = RuntimeService::new();
        let runtime_service_server =
            RuntimeServiceServer::new(runtime_service.clone());
        health_reporter
            .set_serving::<RuntimeServiceServer<RuntimeService>>()
            .await;

        let graceful_shutdown = graceful_shutdown::GracefulShutdown::new(
            health_reporter,
            cell_service,
        );
        let graceful_shutdown_signal = graceful_shutdown.subscribe();

        // Run the server concurrently
        // TODO: pass a known-good path to CellService to store any runtime data.
        let server_handle = tokio::spawn(async move {
            Server::builder()
                .tls_config(tls)?
                .add_service(health_service)
                .add_service(cell_service_server)
                .add_service(discovery_service_server)
                .add_service(observe_service_server)
                // .add_service(pod_service_server)
                .add_service(runtime_service_server)
                .serve_with_incoming_shutdown(socket_stream, async {
                    let mut graceful_shutdown_signal = graceful_shutdown_signal;
                    let _ = graceful_shutdown_signal.changed().await;
                    info!("gRPC server received shutdown signal...");
                })
                .await?;

            info!("gRPC server exited successfully");

            Ok::<_, tonic::transport::Error>(())
        });

        // Event loop
        let graceful_shutdown_handle =
            tokio::spawn(async { graceful_shutdown.wait().await });

        let (server_result, _) =
            tokio::try_join!(server_handle, graceful_shutdown_handle)?;

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
        assert_eq!(AURAE_RUNTIME_DIR, "/var/run/aurae");
    }
}
