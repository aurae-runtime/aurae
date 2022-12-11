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
#![warn(rustdoc::missing_doc_code_examples)]
#![allow(dead_code)]

use anyhow::anyhow;
use anyhow::Context;
use log::*;
use logging::log_channel::LogChannel;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

// Cells
use crate::runtime::CellService;
use aurae_proto::runtime::cell_service_server::CellServiceServer;

pub mod init;
pub mod logging;
mod observe;
mod runtime;
mod schedule;

/// Default Unix domain socket path for `auraed`.
///
/// Warning: This socket is created (by default) with user
/// mode 0o766 which allows for unprivileged access to the
/// auraed daemon which can in turn be used to execute privileged
/// processes and commands. Access to the socket must be governed
/// by an appropriate mTLS Authorization setting in order to maintain
/// a secure multi tenant system.
pub const AURAE_SOCK: &str = "/var/run/aurae/aurae.sock";

pub const AURAE_CELLS_DIR: &str = "/var/run/aurae/cells";

/// Each instance of Aurae holds internal state in memory. Below are the
/// settings which can be configured for a given Aurae daemon instance.
///
/// Note: These fields represent file paths and not the actual authentication
/// material. Each new instance of a subsystem will read these from the local
/// filesystem at runtime in order to authenticate.
#[derive(Debug)]
pub struct AuraedRuntime {
    /// Certificate Authority for an organization or mesh of Aurae instances.
    pub ca_crt: PathBuf,
    /// The signed server X509 certificate for this unique instance.
    pub server_crt: PathBuf,
    /// The secret key for this unique instance.
    pub server_key: PathBuf,
    /// Configurable socket path. Defaults to the value of
    /// `pub const AURAE_SOCK`
    pub socket: PathBuf,
    /// Provides logging channels to expose auraed logging via grpc
    pub log_collector: Arc<LogChannel>,
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

        // Create cells directory
        // TODO Ensure this is dynamic based on runtime flags
        tokio::fs::create_dir_all(AURAE_CELLS_DIR).await.with_context(
            || {
                format!(
                    "Failed to create directory for cells: {}",
                    AURAE_CELLS_DIR
                )
            },
        )?;
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
        let _log_collector = self.log_collector.clone();

        // Run the server concurrently
        let handle = tokio::spawn(async {
            Server::builder()
                .tls_config(tls)?
                .add_service(CellServiceServer::new(CellService::default()))
                // .add_service(ObserveServer::new(ObserveService::new(
                //     log_collector,
                // )))
                .serve_with_incoming(sock_stream)
                .await
        });

        trace!("Setting socket mode {} -> 766", &self.socket.display());

        // We set the mode to 766 for the Unix domain socket.
        // This is what allows non-root users to dial the socket
        // and authenticate with mTLS.
        fs::set_permissions(&self.socket, fs::Permissions::from_mode(0o766))?;
        info!("User Access Socket Created: {}", self.socket.display());

        // Event loop
        handle.await??;
        info!("gRPC server exited successfully");

        Ok(())
    }
}

#[derive(Hash)]
struct CellID {
    base: String,
    command: String,
    timestamp: SystemTime,
}

/// A nondeterministic function used to create a unique ID from a cell name.
/// The same cell name will produce a unique ID based on the time it was
/// created.
fn cell_name_from_string(command: &str) -> Result<String, anyhow::Error> {
    let now = SystemTime::now();
    let mut entries = command.split(' ');
    let base = match entries.next() {
        Some(base) => base,
        None => {
            return Err(anyhow!("empty base command string"));
        }
    };
    let c = CellID {
        base: base.to_string(),
        command: command.to_string(),
        timestamp: now,
    };
    let val = format!("{}-{:?}", base, cell_hash(&c));
    Ok(val)
}

fn cell_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        assert_eq!(AURAE_SOCK, "/var/run/aurae/aurae.sock");
    }
}
