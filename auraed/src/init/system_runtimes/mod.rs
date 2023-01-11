use std::{net::SocketAddr, path::{PathBuf, Path}, os::unix::prelude::PermissionsExt};

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
use super::InitError;
use anyhow::{anyhow, Context};
pub(crate) use cell_system_runtime::CellSystemRuntime;
pub(crate) use container_system_runtime::ContainerSystemRuntime;
pub(crate) use daemon_system_runtime::DaemonSystemRuntime;
pub(crate) use pid1_system_runtime::Pid1SystemRuntime;
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::{TcpListenerStream, UnixListenerStream};
use tonic::async_trait;
use tracing::{trace, info};

mod cell_system_runtime;
mod container_system_runtime;
mod daemon_system_runtime;
mod pid1_system_runtime;

/// A [SocketStream] can represent either a TCP or Unix socket stream.
#[derive(Debug)]
pub enum SocketStream {
    /// Contains a stream for listening over a TCP socket.
    Tcp {
        /// The stream
        stream: TcpListenerStream
    },

    /// Contains a stream for listening over a Unix socket.
    Unix {
        /// The stream
        stream: UnixListenerStream
    },
}

#[async_trait]
pub(crate) trait SystemRuntime {
    async fn init(self, verbose: bool, socket_address: Option<String>) -> Result<SocketStream, InitError>;
}

async fn create_unix_socket_stream(socket_path: PathBuf) -> Result<SocketStream, InitError> {
        let _ = std::fs::remove_file(&socket_path);
        let sock_path = Path::new(&socket_path)
            .parent()
            .ok_or_else(|| anyhow!("unable to find socket path"))?;
        // Create socket directory
        tokio::fs::create_dir_all(sock_path).await.with_context(|| {
            format!(
                "Failed to create directory for socket: {}",
                socket_path.display()
            )
        })?;
        trace!("User Access Socket dir created: {}", sock_path.display());

        let sock = UnixListener::bind(&socket_path)?;

        // We set the mode to 766 for the Unix domain socket.
        // This is what allows non-root users to dial the socket
        // and authenticate with mTLS.
        trace!("Setting socket mode {} -> 766", &socket_path.display());
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o766))?;
        info!("User Access Socket Created: {}", socket_path.display());

        Ok(SocketStream::Unix{stream: UnixListenerStream::new(sock)})
}

async fn create_tcp_socket_stream(socket_addr: SocketAddr) -> Result<SocketStream, InitError> {
    let sock = TcpListener::bind(&socket_addr).await?;
    info!("TCP Access Socket created: {:?}", socket_addr);
    Ok(SocketStream::Tcp{stream: TcpListenerStream::new(sock)})
}

