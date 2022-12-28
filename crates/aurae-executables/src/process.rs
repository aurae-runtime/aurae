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

use nix::sys::signal::{Signal, SIGKILL};
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::io;
use std::io::ErrorKind;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use tracing::info;

#[derive(Debug)]
pub(crate) enum Process {
    Cloned {
        process: procfs::process::Process,
        #[allow(unused)]
        pidfd: OwnedFd,
    },
    Spawned(Child),
}

impl Process {
    pub fn new_from_clone(pid: i32, pidfd: i32) -> io::Result<Self> {
        let process = procfs::process::Process::new(pid)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

        let pidfd = unsafe { OwnedFd::from_raw_fd(pidfd) };

        Ok(Self::Cloned { process, pidfd })
    }

    pub fn new_from_spawn(child: Child) -> Self {
        Self::Spawned(child)
    }

    pub fn kill<T: Into<Option<Signal>>>(
        &mut self,
        signal: T,
    ) -> io::Result<()> {
        let signal = signal.into();
        let pid = match self {
            Process::Cloned { process, .. } => process.pid,
            Process::Spawned(child) => {
                if let Some(SIGKILL) = &signal {
                    // If we are sending a SIGKILL, just use std
                    return child.kill();
                } else {
                    child.id() as i32
                }
            }
        };

        println!("Sending signal ({signal:?}) to pid {pid}");

        nix::sys::signal::kill(Pid::from_raw(pid), signal)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        match self {
            Process::Cloned { process, .. } => {
                let pid = Pid::from_raw(process.pid);

                // https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html
                // The waitpid() function obtains status information for process termination,
                // and optionally process stop and/or continue, from a specified subset of the child processes.
                // If pid is greater than 0, it specifies the process ID of a single child process for which status is requested.
                let exit_status = loop {
                    let WaitStatus::Exited(_, exit_status) = nix::sys::wait::waitpid(pid, None)
                        .map_err(|e| io::Error::from_raw_os_error(e as i32))? else {
                        continue;
                    };

                    break exit_status;
                };

                let exit_status = ExitStatus::from_raw(exit_status);

                info!("Executable with pid {pid} exited with status {exit_status}",);

                Ok(exit_status)
            }
            Process::Spawned(child) => child.wait(),
        }
    }

    pub fn pid(&self) -> Pid {
        match self {
            Process::Cloned { process, .. } => Pid::from_raw(process.pid),
            Process::Spawned(child) => Pid::from_raw(child.id() as i32),
        }
    }
}
