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

use std::path::PathBuf;

use super::{SocketStream, SystemRuntime, SystemRuntimeError};
use crate::init::{
    logging, system_runtimes::create_unix_socket_stream, BANNER,
};
use crate::AURAED_RUNTIME;
use tonic::async_trait;
use tracing::info;

pub(crate) struct ContainerSystemRuntime;

#[async_trait]
impl SystemRuntime for ContainerSystemRuntime {
    async fn init(
        self,
        verbose: bool,
        socket_address: Option<String>,
    ) -> Result<SocketStream, SystemRuntimeError> {
        println!("{BANNER}");
        logging::init(verbose, true)?;
        info!("Running as a container.");
        create_unix_socket_stream(
            socket_address.map(PathBuf::from).unwrap_or_else(|| {
                AURAED_RUNTIME.get().expect("runtime").default_socket_address()
            }),
        )
        .await
    }
}