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

use self::{
    fs::FsError,
    logging::LoggingError,
    network::NetworkError,
    system_runtimes::{
        CellSystemRuntime, ContainerSystemRuntime, Pid1SystemRuntime,
        SystemRuntime,
    },
};

mod fileio;
mod fs;
mod logging;
mod network;
mod power;
mod system_runtimes;

const BANNER: &str = "
    ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
    ┃   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ ┃
    ┃  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ ┃
    ┃  ███████║██║   ██║██████╔╝███████║█████╗   ┃
    ┃  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   ┃
    ┃  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ ┃
    ┃  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ ┃
    ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
         Distributed Systems Runtime Daemon
                 Executing PID 1
    \n";

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
pub async fn init(verbose: bool, nested: bool) {
    let init_result = match Context::get(nested, false) {
        Context::Pid1 => Pid1SystemRuntime {}.init(verbose),
        Context::Cell => CellSystemRuntime {}.init(verbose),
        Context::Container => ContainerSystemRuntime {}.init(verbose),
    }
    .await;

    if let Err(e) = init_result {
        panic!("Failed to initialize: {e:?}")
    }
}

enum Context {
    /// auraed is running as true PID 1
    Pid1,
    /// auraed is nested in a [Cell]
    Cell,
    /// auraed is running in a [Pod] as the init container
    Container,
}

impl Context {
    fn get(nested: bool, container: bool) -> Self {
        // TODO: This is where we need to figure out what the context is without any args.

        if nested {
            Self::Cell
        } else if container {
            Self::Container
        } else if std::process::id() == 1 {
            Self::Pid1
        } else {
            panic!("auraed context could not be determined")
        }
    }
}
