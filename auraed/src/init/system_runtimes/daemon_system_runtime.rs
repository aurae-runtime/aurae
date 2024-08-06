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

use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use super::{SocketStream, SystemRuntime, SystemRuntimeError};
use crate::init::{
    logging,
    system_runtimes::{create_tcp_socket_stream, create_unix_socket_stream},
    BANNER,
};
use crate::AURAED_RUNTIME;
use tonic::async_trait;
use tracing::{info, trace};

pub(crate) struct DaemonSystemRuntime;

#[async_trait]
impl SystemRuntime for DaemonSystemRuntime {
    async fn init(
        self,
        verbose: bool,
        socket_address: Option<String>,
    ) -> Result<SocketStream, SystemRuntimeError> {
        println!("{BANNER}");
        logging::init(verbose, false)?;
        info!("Running as a daemon.");

        // Running as a daemon supports both TCP and Unix sockets for listening, depending on the
        // socket address that's passed in.
        let sockaddr = socket_address.unwrap_or_else(|| {
            AURAED_RUNTIME
                .get()
                .expect("runtime")
                .default_socket_address()
                .to_str()
                .expect("valid default aurae sock path")
                .into()
        });
        if let Ok(addr) = SocketAddr::from_str(&sockaddr) {
            trace!("Listening on TCP: {addr:?}");
            create_tcp_socket_stream(addr).await
        } else {
            trace!("Listening on UNIX: {sockaddr:?}");
            create_unix_socket_stream(PathBuf::from(sockaddr)).await
        }
    }
}