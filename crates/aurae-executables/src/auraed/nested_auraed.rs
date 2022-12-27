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
use crate::process::Process;
use aurae_client::AuraeConfig;
use nix::libc::SIGCHLD;
use nix::unistd::Pid;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitStatus};
use std::{
    io::{self, ErrorKind},
    os::fd::{FromRawFd, OwnedFd},
};

#[derive(Debug)]
pub struct NestedAuraed {
    process: Process,
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

        // Clone docs: https://man7.org/linux/man-pages/man2/clone.2.html

        // If this signal is specified as anything other than SIGCHLD, then the
        //        parent process must specify the __WALL or __WCLONE options when
        //        waiting for the child with wait(2).  If no signal (i.e., zero) is
        //        specified, then the parent process is not signaled when the child
        //        terminates.
        let signal = SIGCHLD;

        let mut pid_fd = -1;
        let mut clone = clone3::Clone3::default();
        clone.flag_pidfd(&mut pid_fd);
        clone.flag_vfork();
        clone.exit_signal(signal as u64);

        // If CLONE_NEWNS is set, the cloned child is started in a
        // new mount namespace, initialized with a copy of the
        // namespace of the parent.  If CLONE_NEWNS is not set, the
        // child lives in the same mount namespace as the parent.
        if !shared_namespaces.mount {
            clone.flag_newns();
        }

        //If CLONE_NEWUTS is set, then create the process in a new
        // UTS namespace, whose identifiers are initialized by
        // duplicating the identifiers from the UTS namespace of the
        // calling process.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same UTS namespace
        // as the calling process.
        if !shared_namespaces.uts {
            clone.flag_newuts();
        }

        // If CLONE_NEWIPC is set, then create the process in a new
        // IPC namespace.  If this flag is not set, then (as with
        // fork(2)), the process is created in the same IPC namespace
        // as the calling process.
        if !shared_namespaces.ipc {
            clone.flag_newipc();
        }

        // If CLONE_NEWPID is set, then create the process in a new
        // PID namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same PID namespace
        // as the calling process.
        if !shared_namespaces.pid {
            clone.flag_newpid();
        }

        // If CLONE_NEWNET is set, then create the process in a new
        // network namespace.  If this flag is not set, then (as with
        // fork(2)) the process is created in the same network
        // namespace as the calling process.
        if !shared_namespaces.net {
            clone.flag_newnet();
        }

        // If this flag is not set, then (as with fork(2)) the process is
        // created in the same cgroup namespaces as the calling
        // process.
        if !shared_namespaces.cgroup {
            clone.flag_newcgroup();
        }

        // We have a concern that the "command" API make change/break in the future and this
        // test is intended to help safeguard against that!
        // We check that the command we kept has the expected number of args following the call
        // to command.args, whose return value we ignored above.
        assert_eq!(command.get_args().len(), 3);

        // NOTE: AFTER THIS CALL YOU CAN BE IN THE CURRENT OR CHILD PROCESS.
        let pid = unsafe { clone.call() }
            .map_err(|e| io::Error::from_raw_os_error(e.0))?;

        if pid == 0 {
            // we are in the child

            let command = {
                let shared_namespaces = shared_namespaces.clone();
                unsafe {
                    command.pre_exec(move || {
                        if !shared_namespaces.mount {
                            // We can potentially do some setup here, and then leave the rest
                            // to auraed's init. We would need flags to signal it is a nested PID1

                            // remount as private
                            nix::mount::mount(
                                None::<&str>, // ignored
                                ".",
                                None::<&str>, // ignored
                                nix::mount::MsFlags::MS_SLAVE
                                    | nix::mount::MsFlags::MS_REC,
                                None::<&str>, // ignored
                            )
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

            command.exec();
        }

        // we are in the parent again
        let process = procfs::process::Process::new(pid)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

        let pid_fd = unsafe { OwnedFd::from_raw_fd(pid_fd) };

        Ok(Self {
            process: Process {
                inner: process,
                pid_fd: Some(pid_fd),
                child: None,
            },
            client_config,
        })
    }

    pub fn kill(&self) -> io::Result<ExitStatus> {
        self.process.kill()
    }

    pub fn pid(&self) -> Pid {
        Pid::from_raw(self.process.pid)
    }
}
