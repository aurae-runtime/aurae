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

use super::SharedNamespaces;
use aurae_client::AuraeConfig;
use nix::{
    libc::SIGCHLD,
    mount::MntFlags,
    sys::signal::{Signal, SIGKILL, SIGTERM},
    unistd::Pid,
};
use std::{
    io::{self, ErrorKind},
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Command, ExitStatus},
};
use tracing::trace;

#[derive(Debug)]
pub struct NestedAuraed {
    process: procfs::process::Process,
    #[allow(unused)]
    pidfd: i32,
    shared_namespaces: SharedNamespaces,
    pub client_config: AuraeConfig,
}

impl NestedAuraed {
    pub fn new(shared_namespaces: SharedNamespaces) -> io::Result<Self> {
        // Launch nested Auraed
        //
        // Here we launch a nested auraed with the --nested flag
        // which is used our way of "hooking" into the newly created
        // aurae isolation zone.
        //
        // TODO: Consider changing "--nested" to "--nested-cell" or similar
        // TODO: handle expect
        let mut client_config =
            AuraeConfig::try_default().expect("file based config");
        client_config.system.socket =
            format!("/var/run/aurae/aurae-{}.sock", uuid::Uuid::new_v4());

        let mut command = Command::new("auraed");
        let _ = command.current_dir("/").args([
            "--socket",
            &client_config.system.socket,
            "--nested",
        ]);

        // We have a concern that the "command" API make change/break in the future and this
        // test is intended to help safeguard against that!
        // We check that the command we kept has the expected number of args following the call
        // to command.args, whose return value we ignored above.
        assert_eq!(command.get_args().len(), 3);

        // Clone docs: https://man7.org/linux/man-pages/man2/clone.2.html

        // If this signal is specified as anything other than SIGCHLD, then the
        //        parent process must specify the __WALL or __WCLONE options when
        //        waiting for the child with wait(2).  If no signal (i.e., zero) is
        //        specified, then the parent process is not signaled when the child
        //        terminates.
        let signal = SIGCHLD;

        let mut pidfd = -1;
        let mut clone = clone3::Clone3::default();
        let _ = clone.flag_pidfd(&mut pidfd);
        let _ = clone.flag_vfork();
        let _ = clone.exit_signal(signal as u64);

        // Note: The logic here is reversed. We define the flags as "share'
        //       and map them to "unshare".
        //       This is by design as the API has a concept of "share".

        // If CLONE_NEWNS is set, the cloned child is started in a
        // new mount namespace, initialized with a copy of the
        // namespace of the parent.  If CLONE_NEWNS is not set, the
        // child lives in the same mount namespace as the parent.
        if !shared_namespaces.mount {
            let _ = clone.flag_newns();
        }

        //If CLONE_NEWUTS is set, then create the process in a new
        // UTS namespace, whose identifiers are initialized by
        // duplicating the identifiers from the UTS namespace of the
        // calling process.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same UTS namespace
        // as the calling process.
        if !shared_namespaces.uts {
            let _ = clone.flag_newuts();
        }

        // If CLONE_NEWIPC is set, then create the process in a new
        // IPC namespace.  If this flag is not set, then (as with
        // fork(2)), the process is created in the same IPC namespace
        // as the calling process.
        if !shared_namespaces.ipc {
            let _ = clone.flag_newipc();
        }

        // If CLONE_NEWPID is set, then create the process in a new
        // PID namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same PID namespace
        // as the calling process.
        if !shared_namespaces.pid {
            let _ = clone.flag_newpid();
        }

        // If CLONE_NEWNET is set, then create the process in a new
        // network namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same network
        // namespace as the calling process.
        if !shared_namespaces.net {
            let _ = clone.flag_newnet();
        }

        // If this flag is not set, then (as with fork(2)) the process is
        // created in the same cgroup namespaces as the calling
        // process.
        if !shared_namespaces.cgroup {
            let _ = clone.flag_newcgroup();
        }

        // TODO: clone uses the same pattern as command. Safeguard against changes

        // NOTE: AFTER THIS CALL YOU CAN BE IN THE CURRENT OR CHILD PROCESS.
        let pid = unsafe { clone.call() }
            .map_err(|e| io::Error::from_raw_os_error(e.0))?;

        if pid == 0 {
            // we are in the child

            let command = {
                unsafe {
                    command.pre_exec(move || {
                        // We can do the steps for isolation here and leave the rest to
                        // auraed's init. This would probably require sending the
                        // shared_namespaces data in the args.

                        if !shared_namespaces.mount {
                            // remount as private
                            nix::mount::mount(
                                None::<&str>, // ignored
                                ".",
                                None::<&str>, // ignored
                                nix::mount::MsFlags::MS_PRIVATE
                                    | nix::mount::MsFlags::MS_REC,
                                None::<&str>, // ignored
                            )
                            .map_err(|e| {
                                io::Error::from_raw_os_error(e as i32)
                            })?;
                        }

                        if wants_isolated_pid(&shared_namespaces) {
                            nix::mount::umount2("/proc", MntFlags::MNT_DETACH)
                                .map_err(|e| {
                                    io::Error::from_raw_os_error(e as i32)
                                })?;

                            nix::mount::mount(
                                Some("proc"),
                                "/proc",
                                Some("proc"),
                                nix::mount::MsFlags::empty(),
                                None::<&[u8]>,
                            )
                            .map_err(|e| {
                                io::Error::from_raw_os_error(e as i32)
                            })?;
                        }

                        Ok(())
                    })
                }
            };

            // TODO: check that exec should never return, even after exit
            let e = command.exec();
            return Err(e);
        }

        let process = procfs::process::Process::new(pid)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

        Ok(Self { process, pidfd, shared_namespaces, client_config })
    }

    /// Sends a graceful shutdown signal to the auraed process.
    pub fn shutdown(&mut self) -> io::Result<ExitStatus> {
        // TODO: Here, SIGTERM works when using auraescript, but hangs(?) during unit tests.
        //       SIGKILL, however, works. The hang is avoided if all namespaces are shared.
        //       Tests have not been done to figure out which namespace is the cause of the hang.
        self.do_kill(Some(SIGTERM))?;
        self.wait()
    }

    /// Sends a [SIGKILL] signal to the auraed process.
    pub fn kill(&mut self) -> io::Result<ExitStatus> {
        self.do_kill(Some(SIGKILL))?;
        self.wait()
    }

    fn do_kill<T: Into<Option<Signal>>>(
        &mut self,
        signal: T,
    ) -> io::Result<()> {
        let signal = signal.into();
        let pid = Pid::from_raw(self.process.pid);

        nix::sys::signal::kill(pid, signal)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        let pid = Pid::from_raw(self.process.pid);

        let mut exit_status = 0;
        let _child_pid = loop {
            let res =
                unsafe { libc::waitpid(pid.as_raw(), &mut exit_status, 0) };

            if res == -1 {
                let err = io::Error::last_os_error();
                match err.kind() {
                    ErrorKind::Interrupted => continue,
                    _ => break Err(err),
                }
            }

            break Ok(res);
        }?;

        let exit_status = ExitStatus::from_raw(exit_status);

        trace!("Pid {pid} exited with status {exit_status}");

        Ok(exit_status)
    }

    pub fn pid(&self) -> Pid {
        Pid::from_raw(self.process.pid)
    }
}

fn wants_isolated_pid(shared_namespaces: &SharedNamespaces) -> bool {
    // TODO: This is wrong. A process wants an isolated pid namespace, regardless of mount
    !shared_namespaces.pid && !shared_namespaces.mount
}

// On cleaning up other files these todos were there.
// We may have avoided the need for tracking namespaces,
// but I (future-highway) don't want to delete these until we are sure.
// https://github.com/aurae-runtime/aurae/issues/200#issuecomment-1366279569
// // TODO We need to track the namespace for all newly
// //      unshared namespaces within a Cell such that
// //      we can call command.set_namespace() for
// //      each of the new namespaces at the cell level!
// //      This will likely require changing how Cells
// //      manage namespaces as we need to cache the namespace
// //      IDs (names?)
// //
// // TODO Basically once a namespace has been created for a Cell
// //      we should put ALL future executables into the same namespace!
