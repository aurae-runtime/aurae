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
        CellSystemRuntime, ContainerSystemRuntime, DaemonSystemRuntime,
        Pid1SystemRuntime, SystemRuntime,
    },
};
pub use self::system_runtimes::SocketStream;
use std::fs::File;
use std::io::{BufReader, Read};
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
         Distributed Systems Runtime Daemon\n";

#[derive(thiserror::Error, Debug)]
pub(crate) enum InitError {
    #[error(transparent)]
    Logging(#[from] LoggingError),
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),
}

/// Initialize aurae, depending on our context.
pub async fn init(verbose: bool, nested: bool, socket_address: Option<String>) -> SocketStream {
    let init_result = match Context::get(nested) {
        Context::Pid1 => Pid1SystemRuntime {}.init(verbose, socket_address),
        Context::Cell => CellSystemRuntime {}.init(verbose, socket_address),
        Context::Container => ContainerSystemRuntime {}.init(verbose, socket_address),
        Context::Daemon => DaemonSystemRuntime {}.init(verbose, socket_address),
    }
    .await;

    if let Err(e) = init_result {
        panic!("Failed to initialize: {:?}", e)
    }
    init_result.expect("this is impossible as we check for error just above")
}

enum Context {
    /// auraed is running as true PID 1
    Pid1,
    /// auraed is nested in a [Cell]
    Cell,
    /// auraed is running in a [Pod] as the init container
    Container,
    /// auraed is running as [Daemon] or arbitrarily on a host
    Daemon,
}

impl Context {
    fn get(nested: bool) -> Self {
        // TODO: Manage nested bool without passing --nested
        let in_c = in_new_cgroup_namespace();
        if in_c && !nested {
            // If we are in a container, we should always run this setup no matter pid 1 or not
            return Self::Container;
        }
        if std::process::id() == 1 {
            return Self::Pid1;
        }
        if nested {
            Self::Cell
        } else {
            Self::Daemon
        }
    }
}

// Here we have bespoke "in_container" logic that will check and see if we
// are executing inside an Aurae pod container.
//
// Auraed container: /proc/self/cgroup: 0::/
// Auraed cell     : /proc/self/cgroup: 0::/../../../ae-1/_
// Systemd init    : /proc/self/cgroup: 0::/init.scope
// User slice      : /proc/self/cgroup: 0::/user.slice/user-1000.slice/session-3.scope
//
//        When reading the cgroup memberships of a "target" process from
//        /proc/[pid]/cgroup, the pathname shown in the third field of each
//        record will be relative to the reading process's root directory
//        for the corresponding cgroup hierarchy.  If the cgroup directory
//        of the target process lies outside the root directory of the
//        reading process's cgroup namespace, then the pathname will show
//        ../ entries for each ancestor level in the cgroup hierarchy.
//
// Source: https://man7.org/linux/man-pages/man7/cgroup_namespaces.7.html
fn in_new_cgroup_namespace() -> bool {
    let file =
        File::open("/proc/self/cgroup").expect("opening /proc/self/cgroup");
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    let _ = reader
        .read_to_string(&mut contents)
        .expect("reading /proc/self/cgroup");

    // Here we examine the last few bytes of /proc/self/cgroup
    // We know if the cgroup string ends with a \n newline
    // as well as a / as in "0::/" we are in a new (and nested)
    // cgroup namespace.
    //
    // For all intents and purposes this is the closest way we
    // can guarantee that we are in "a container".
    //
    // It is important to note that Aurae cells (by default)
    // will also schedule themselves in a new cgroup namespace.
    // Therefore we would expect Aurae cells to also match this
    // pattern.
    //
    contents.to_string().ends_with("/\n")
}
