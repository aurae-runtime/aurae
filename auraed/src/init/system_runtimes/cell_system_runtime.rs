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

use std::path::PathBuf;

use super::{SocketStream, SystemRuntime, SystemRuntimeError};
use crate::{
    init::{logging, system_runtimes::create_unix_socket_stream, BANNER},
    AURAE_SOCK, AURAE_RUNTIME_DIR,
};
use tonic::async_trait;
use tracing::info;

pub(crate) struct CellSystemRuntime;

#[async_trait]
impl SystemRuntime for CellSystemRuntime {
    async fn init(
        self,
        verbose: bool,
        socket_address: Option<String>,
    ) -> Result<SocketStream, SystemRuntimeError> {
        println!("{}", BANNER);
        logging::init(verbose, false)?;
        info!("Running as a cell");
        let mut default_aurae_sock_path = PathBuf::from(AURAE_RUNTIME_DIR);
        default_aurae_sock_path.push(AURAE_SOCK);
        create_unix_socket_stream(
            socket_address
                .map(PathBuf::from)
                .unwrap_or_else(|| default_aurae_sock_path),
        )
        .await
    }
}
