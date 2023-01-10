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
}

/// Run Aurae as an init pid 1 instance.
pub async fn init(verbose: bool, nested: bool) {
    let init_result = match Context::get(nested) {
        Context::Pid1 => Pid1SystemRuntime {}.init(verbose),
        Context::Cell => CellSystemRuntime {}.init(verbose),
        Context::Container => ContainerSystemRuntime {}.init(verbose),
        Context::Daemon => DaemonSystemRuntime {}.init(verbose),
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
    /// auraed is running as [Daemon] or arbitrarily on a host
    Daemon,
}

impl Context {
    fn get(nested: bool) -> Self {
        // TODO: Manage nested bool without passing --nested
        let in_c = in_container();
        if nested {
            Self::Cell
        } else if std::process::id() == 1 {
            if in_c {
                Self::Container
            } else {
                Self::Pid1
            }
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
fn in_container() -> bool {
    let file =
        File::open("/proc/self/cgroup").expect("opening /proc/self/cgroup");
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    let _ = reader
        .read_to_string(&mut contents)
        .expect("reading /proc/self/cgroup");
    contents.to_string().ends_with('/')
}
