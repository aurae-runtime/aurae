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

use super::{shared_namespaces::Unshare, SharedNamespaces};
use aurae_client::AuraeConfig;
use nix::{
    libc::SIGCHLD,
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

        let random = uuid::Uuid::new_v4();

        // TODO: handle expect
        let mut client_config =
            AuraeConfig::try_default().expect("file based config");
        client_config.system.socket =
            format!("/var/run/aurae/aurae-{random}.sock");

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

        let signal = SIGCHLD;

        let mut pidfd = -1;
        let mut clone = clone3::Clone3::default();
        let _ = clone.flag_pidfd(&mut pidfd);
        let _ = clone.flag_vfork();
        let _ = clone.exit_signal(signal as u64);

        // Note: The logic here is reversed. We define the flags as "share'
        //       and map them to "unshare".
        //       This is by design as the API has a concept of "share".

        // Order: mount, uts, ipc, pid, network, user (don't have), cgroup

        let mut unshare = Unshare::default();
        unshare.prep(&shared_namespaces)?;

        if !shared_namespaces.mount {
            let _ = clone.flag_newns();
        }

        if !shared_namespaces.uts {
            let _ = clone.flag_newuts();
        }

        if !shared_namespaces.ipc {
            let _ = clone.flag_newipc();
        }

        if !shared_namespaces.pid {
            let _ = clone.flag_newpid();
        }

        if !shared_namespaces.net {
            let _ = clone.flag_newnet();
        }

        if !shared_namespaces.cgroup {
            let _ = clone.flag_newcgroup();
        }

        // TODO: clone uses the same pattern as command. Safeguard against changes

        match unsafe { clone.call() }
            .map_err(|e| io::Error::from_raw_os_error(e.0))?
        {
            0 => {
                // child
                let command = {
                    unsafe {
                        command.pre_exec(move || {
                            unshare.mount_namespace_unmount_with_exceptions(
                                &shared_namespaces,
                            )?;
                            unshare.uts_namespace(&shared_namespaces)?;
                            unshare.ipc_namespace(&shared_namespaces)?;
                            unshare.pid_namespace(&shared_namespaces)?;
                            unshare.net_namespace(&shared_namespaces)?;
                            unshare.cgroup_namespace(&shared_namespaces)?;

                            Ok(())
                        })
                    }
                };

                // TODO: check that exec should never return, even after exit
                let e = command.exec();
                println!("{e:#?}");
                Err(e)
            }
            pid => {
                // parent
                println!("Nested auraed has pid {pid}");

                let process = procfs::process::Process::new(pid)
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

                Ok(Self { process, pidfd, shared_namespaces, client_config })
            }
        }
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
