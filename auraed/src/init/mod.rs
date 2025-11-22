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
    let init_result = init_with_runtimes(
        context,
        verbose,
        socket_address,
        Pid1SystemRuntime {},
        CellSystemRuntime {},
        ContainerSystemRuntime {},
        DaemonSystemRuntime {},
    )
    .await;

    match init_result {
        Ok(stream) => (context, stream),
        Err(e) => panic!("Failed to initialize: {e:?}"),
    }
}

async fn init_with_runtimes<RPid1, RCell, RContainer, RDaemon>(
    context: Context,
    verbose: bool,
    socket_address: Option<String>,
    pid1_runtime: RPid1,
    cell_runtime: RCell,
    container_runtime: RContainer,
    daemon_runtime: RDaemon,
) -> Result<SocketStream, SystemRuntimeError>
where
    RPid1: SystemRuntime,
    RCell: SystemRuntime,
    RContainer: SystemRuntime,
    RDaemon: SystemRuntime,
{
    match context {
        Context::Pid1 => pid1_runtime.init(verbose, socket_address).await,
        Context::Cell => cell_runtime.init(verbose, socket_address).await,
        Context::Container => {
            container_runtime.init(verbose, socket_address).await
        }
        Context::Daemon => daemon_runtime.init(verbose, socket_address).await,
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
    use crate::init::system_runtimes::{
        SocketStream, SystemRuntime, SystemRuntimeError,
    };
    use anyhow::anyhow;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::runtime::Runtime;
    use tonic::async_trait;

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

    #[test]
    fn context_get_prefers_container_when_in_cgroup_namespace() {
        assert_eq!(
            Context::get_with_detectors(
                false,
                ContextDetectors {
                    pid_fn: pid_42,
                    in_cgroup_fn: in_cgroup_true
                }
            ),
            Context::Container
        );
        assert_eq!(
            Context::get_with_detectors(
                true,
                ContextDetectors {
                    pid_fn: pid_42,
                    in_cgroup_fn: in_cgroup_true
                }
            ),
            Context::Cell
        );
    }

    #[derive(Clone)]
    struct MockRuntime {
        calls: Arc<AtomicUsize>,
        label: &'static str,
    }

    impl MockRuntime {
        fn new(label: &'static str) -> Self {
            Self { calls: Arc::new(AtomicUsize::new(0)), label }
        }
    }

    #[async_trait]
    impl SystemRuntime for Arc<MockRuntime> {
        async fn init(
            self,
            _verbose: bool,
            _socket_address: Option<String>,
        ) -> Result<SocketStream, SystemRuntimeError> {
            let _ = self.calls.fetch_add(1, Ordering::SeqCst);
            Err(SystemRuntimeError::Other(anyhow!(self.label)))
        }
    }

    fn assert_called_once(mock: &Arc<MockRuntime>) {
        assert_eq!(
            mock.calls.load(Ordering::SeqCst),
            1,
            "expected {} to be called once",
            mock.label
        );
    }

    #[test]
    fn init_should_call_matching_system_runtime() {
        // This test ensures the `init` dispatcher chooses the correct runtime
        // implementation for each Context. We avoid spinning up real runtimes
        // by injecting cheap mocks that count how many times they're called.
        let rt = Runtime::new().expect("tokio runtime");

        let pid1 = Arc::new(MockRuntime::new("pid1"));
        let cell = Arc::new(MockRuntime::new("cell"));
        let container = Arc::new(MockRuntime::new("container"));
        let daemon = Arc::new(MockRuntime::new("daemon"));

        rt.block_on(async {
            // Each tuple represents (nested flag, pid, in_cgroup_namespace).
            // We exercise the four Context variants the same way Context::get does.
            let runtimes = [
                (false, 1, false),
                (true, 1, false),
                (false, 42, true),
                (false, 42, false),
            ];

            for (nested, pid, in_cgroup) in runtimes {
                let ctx = derive_context(nested, pid, in_cgroup);

                // Call the same routing code init() uses, but with our mocks.
                let _ = init_with_runtimes(
                    ctx,
                    false,
                    None,
                    pid1.clone(),
                    cell.clone(),
                    container.clone(),
                    daemon.clone(),
                )
                .await;
            }
        });

        // Each mock should have been called exactly once by its matching Context.
        assert_called_once(&pid1);
        assert_called_once(&cell);
        assert_called_once(&container);
        assert_called_once(&daemon);
    }
}
