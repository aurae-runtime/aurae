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

//! Run the Aurae daemon as a pid 1 init program.
//!
//! The Aurae daemon assumes that if the current process id (PID) is 1 to
//! run itself as an initialization program, otherwise bypass the init module.

use crate::init::{
    fs::FsError,
    logging::LoggingError,
    network::NetworkError,
    system_runtime::{Pid1SystemRuntime, PidGt1SystemRuntime, SystemRuntime},
};
use tracing::Level;

mod fileio;
mod fs;
mod logging;
mod network;
mod power;
mod system_runtime;

const BANNER: &str = "
    +--------------------------------------------+
    |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |
    |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |
    |  ███████║██║   ██║██████╔╝███████║█████╗   |
    |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |
    |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |
    |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |
    +--------------------------------------------+\n";

#[derive(thiserror::Error, Debug)]
pub(crate) enum InitError {
    #[error(transparent)]
    Logging(#[from] LoggingError),
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Network(#[from] NetworkError),
}

/// Run Aurae as an init pid 1 instance.
pub async fn init(logger_level: Level, nested: bool) {
    let res = match (std::process::id(), nested) {
        (0, _) => unreachable!(
            "process is running as PID 0, which should be impossible"
        ),
        (1, false) => Pid1SystemRuntime {}.init(logger_level),
        _ => PidGt1SystemRuntime {}.init(logger_level),
    }
    .await;

    if let Err(e) = res {
        panic!("Failed to initialize: {e:?}")
    }
}
