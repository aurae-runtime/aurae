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

//! Run the Aurae daemon as a pid 1 init program.
//!
//! The Aurae daemon assumes that if the current process id (PID) is 1 to
//! run itself as an initialization program, otherwise bypass the init module.

pub use self::system_runtimes::SocketStream;
use self::system_runtimes::{
    CellSystemRuntime, ContainerSystemRuntime, DaemonSystemRuntime,
    Pid1SystemRuntime, SystemRuntime, SystemRuntimeError,
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

         Distributed Systems Runtime Daemon

 ┌───────────────────────────────────────────────────┐
 │  WARNING WARNING WARNING WARNING WARNING WARNING  │
 │                                                   │
 │ The Aurae Runtime Project is currently in a state │
 │ of 'Early Active Development'. The current APIs   │
 │ and features of the project should be considered  │
 │ unstable. As the project matures the APIs and     │
 │ features will stabilize.                          │
 │                                                   │
 │ As the project maintainers deem appropriate the   │
 │ project will remove this warning.                 │
 │                                                   │
 │ At the time this banner is removed the project    │
 │ will have documentation available in the main     │
 │ repository on current API stability and backwards │
 │ compatability.                                    │
 │                                                   │
 │          github.com/aurae-runtime/aurae           │
 │                                                   │
 │  WARNING WARNING WARNING WARNING WARNING WARNING  │
 └───────────────────────────────────────────────────┘
\n";

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub(crate) enum InitError {
    #[error(transparent)]
    SystemRuntimeError(#[from] SystemRuntimeError),
}

/// Initialize aurae, depending on our context.
pub async fn init(
    verbose: bool,
    nested: bool,
    socket_address: Option<String>,
) -> (Context, SocketStream) {
    let context = Context::get(nested);
    let init_result = match context {
        Context::Pid1 => Pid1SystemRuntime {}.init(verbose, socket_address),
        Context::Cell => CellSystemRuntime {}.init(verbose, socket_address),
        Context::Container => {
            ContainerSystemRuntime {}.init(verbose, socket_address)
        }
        Context::Daemon => DaemonSystemRuntime {}.init(verbose, socket_address),
    }
    .await;

    match init_result {
        Ok(stream) => (context, stream),
        Err(e) => panic!("Failed to initialize: {e:?}"),
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Context {
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
    pub fn get(nested: bool) -> Self {
        // TODO: Manage nested bool without passing --nested
        Self::get_with_detectors(nested, ContextDetectors::default())
    }

    fn get_with_detectors(nested: bool, detectors: ContextDetectors) -> Self {
        let pid = (detectors.pid_fn)();
        let in_cgroup = (detectors.in_cgroup_fn)();
        derive_context(nested, pid, in_cgroup)
    }
}

#[derive(Clone, Copy)]
struct ContextDetectors {
    pid_fn: fn() -> u32,
    in_cgroup_fn: fn() -> bool,
}

impl Default for ContextDetectors {
    fn default() -> Self {
        ContextDetectors {
            pid_fn: std::process::id,
            in_cgroup_fn: in_new_cgroup_namespace,
        }
    }
}

fn derive_context(
    nested: bool,
    pid: u32,
    in_cgroup_namespace: bool,
) -> Context {
    if in_cgroup_namespace && !nested {
        // If we are in a container, we should always run this setup no matter pid 1 or not
        Context::Container
    } else if nested {
        // If we are nested, we should always run this setup no matter pid 1 or not
        Context::Cell
    } else if pid == 1 {
        Context::Pid1
    } else {
        Context::Daemon
    }
}

// Here we have bespoke "in_container" logic that will check and see if we
// are executing inside an Aurae pod container.
//
// Note: All of the contents of the "cgroup" files in procfs end with a trailing \n newline byte
//
// Auraed container: /proc/self/cgroup: 0::/_aurae
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
    let file = File::open("/proc/self/cgroup");

    // Note: The following is a workaround for a chicken egg problem in the init
    //       logic. We need to read from /proc to determine whether we're in a
    //       container or whether we're running as true PID 1. But if we're
    //       running as true PID 1, /proc wouldn't be mounted at this point as
    //       we only mount proc when we have determined that we _are_ running as
    //       true PID 1.
    match file {
        Ok(file) => {
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
            contents.to_string().ends_with("_aurae\n")
            // TODO Use the AURAE_SELF_IDENTIFIER const as currently defined in runtime_service.rs
            // TODO Consider moving the const to a better home :)
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid_one() -> u32 {
        1
    }

    fn pid_42() -> u32 {
        42
    }

    fn in_cgroup_true() -> bool {
        true
    }

    fn in_cgroup_false() -> bool {
        false
    }

    #[test]
    fn context_get_should_respect_pid_and_nested_flag() {
        type PidFn = fn() -> u32;
        type CgroupFn = fn() -> bool;

        let cases: &[(bool, PidFn, CgroupFn, Context)] = &[
            (false, pid_one, in_cgroup_false, Context::Pid1),
            (true, pid_one, in_cgroup_false, Context::Cell),
            (false, pid_42, in_cgroup_true, Context::Container),
            (true, pid_42, in_cgroup_true, Context::Cell),
            (false, pid_42, in_cgroup_false, Context::Daemon),
        ];

        for &(nested, pid_fn, in_cgroup_fn, expected) in cases {
            assert_eq!(
                Context::get_with_detectors(
                    nested,
                    ContextDetectors { pid_fn, in_cgroup_fn }
                ),
                expected,
                "nested={nested} pid_fn_ptr={:p} in_cgroup_fn_ptr={:p}",
                pid_fn as *const (),
                in_cgroup_fn as *const ()
            );
        }
    }
}
