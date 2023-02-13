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

use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use super::{SocketStream, SystemRuntime, SystemRuntimeError};
use crate::init::{
    logging,
    system_runtimes::{create_tcp_socket_stream, create_unix_socket_stream},
    AURAE_SOCK, BANNER,
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
                .runtime_dir
                .join(AURAE_SOCK)
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
