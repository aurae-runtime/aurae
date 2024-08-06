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
use anyhow::{anyhow, Context};
pub(crate) use cell_system_runtime::CellSystemRuntime;
pub(crate) use container_system_runtime::ContainerSystemRuntime;
pub(crate) use daemon_system_runtime::DaemonSystemRuntime;
pub(crate) use pid1_system_runtime::Pid1SystemRuntime;
use std::{
    net::SocketAddr,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::{TcpListenerStream, UnixListenerStream};
use tonic::async_trait;
use tracing::{info, trace};

use super::{fs::FsError, logging::LoggingError, network::NetworkError};

mod cell_system_runtime;
mod container_system_runtime;
mod daemon_system_runtime;
mod pid1_system_runtime;

#[derive(thiserror::Error, Debug)]
pub(crate) enum SystemRuntimeError {
    #[error(transparent)]
    FsError(#[from] FsError),
    #[error(transparent)]
    Logging(#[from] LoggingError),
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// A [SocketStream] can represent either a TCP or Unix socket stream.
#[derive(Debug)]
pub enum SocketStream {
    /// Contains a stream for listening over a TCP socket.
    Tcp(TcpListenerStream),

    /// Contains a stream for listening over a Unix socket.
    Unix(UnixListenerStream),
}

#[async_trait]
pub(crate) trait SystemRuntime {
    async fn init(
        self,
        verbose: bool,
        socket_address: Option<String>,
    ) -> Result<SocketStream, SystemRuntimeError>;
}

async fn create_unix_socket_stream(
    socket_path: PathBuf,
) -> Result<SocketStream, SystemRuntimeError> {
    let _ = std::fs::remove_file(&socket_path);
    let sock_path = Path::new(&socket_path).parent().ok_or_else(|| {
        anyhow!("not a valid socket path: {:?}", &socket_path)
    })?;
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
    std::fs::set_permissions(
        &socket_path,
        std::fs::Permissions::from_mode(0o766),
    )?;
    info!("User Access Socket Created: {}", socket_path.display());

    Ok(SocketStream::Unix(UnixListenerStream::new(sock)))
}

async fn create_tcp_socket_stream(
    socket_addr: SocketAddr,
) -> Result<SocketStream, SystemRuntimeError> {
    trace!("creating tcp stream for {:?}", socket_addr);
    let sock = TcpListener::bind(&socket_addr).await?;
    info!("TCP Access Socket created: {:?}", socket_addr);
    Ok(SocketStream::Tcp(TcpListenerStream::new(sock)))
}