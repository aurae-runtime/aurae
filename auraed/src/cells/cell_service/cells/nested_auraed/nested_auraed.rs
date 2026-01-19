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

use super::isolation_controls::{Isolation, IsolationControls};
use crate::AURAED_RUNTIME;
use client::AuraeSocket;
use clone3::Flags;
use nix::{
    errno::Errno,
    libc::SIGCHLD,
    sys::{
        signal::{Signal, Signal::SIGKILL, Signal::SIGTERM},
        wait::{WaitStatus, waitpid},
    },
    unistd::Pid,
};
use std::path::PathBuf;
use std::{
    io,
    os::unix::process::{CommandExt, ExitStatusExt},
    process::{Command, ExitStatus},
};
use tracing::{error, info, trace};

#[derive(Debug)]
pub struct NestedAuraed {
    process: procfs::process::Process,
    #[allow(unused)]
    pidfd: i32,
    #[allow(unused)]
    iso_ctl: IsolationControls,
    pub client_socket: AuraeSocket,
}

impl NestedAuraed {
    pub fn new(name: String, iso_ctl: IsolationControls) -> io::Result<Self> {
        // Here we launch a nested auraed with the --nested flag
        // which is used our way of "hooking" into the newly created
        // aurae isolation zone.

        let auraed_runtime = AURAED_RUNTIME.get().expect("runtime");

        let socket_path = format!(
            "{}/aurae-{}.sock",
            auraed_runtime.runtime_dir.to_string_lossy(),
            uuid::Uuid::new_v4(),
        );

        let client_socket = AuraeSocket::Path(socket_path.clone().into());

        let auraed_path: PathBuf =
            auraed_runtime.auraed.clone().try_into().expect("path to auraed");
        let mut command = Command::new(auraed_path);

        let _ = command.args([
            "--socket",
            &socket_path,
            "--nested", // NOTE: for now, the nested flag only signals for the code in the init module to not trigger (i.e., don't run the pid 1 code, run the non pid 1 code)
            "--server-crt",
            &auraed_runtime.server_crt.to_string_lossy(),
            "--server-key",
            &auraed_runtime.server_key.to_string_lossy(),
            "--ca-crt",
            &auraed_runtime.ca_crt.to_string_lossy(),
            "--runtime-dir",
            &auraed_runtime.runtime_dir.to_string_lossy(),
            "--library-dir",
            &auraed_runtime.library_dir.to_string_lossy(),
        ]);

        // We have a concern that the "command" API make change/break in the future and this
        // test is intended to help safeguard against that!
        // We check that the command we kept has the expected number of args following the call
        // to command.args, whose return value we ignored above.
        assert_eq!(command.get_args().len(), 13);

        // *****************************************************************
        // ██████╗██╗      ██████╗ ███╗   ██╗███████╗██████╗
        // ██╔════╝██║     ██╔═══██╗████╗  ██║██╔════╝╚════██╗
        // ██║     ██║     ██║   ██║██╔██╗ ██║█████╗   █████╔╝
        // ██║     ██║     ██║   ██║██║╚██╗██║██╔══╝   ╚═══██╗
        // ╚██████╗███████╗╚██████╔╝██║ ╚████║███████╗██████╔╝
        // ╚═════╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═════╝
        // Clone docs: https://man7.org/linux/man-pages/man2/clone.2.html
        // *****************************************************************

        // Prepare clone3 command to "execute" the nested auraed
        let mut clone = clone3::Clone3::default();

        // [ Options ]

        // If the child fails to start, indicate an error
        // Set the pid file descriptor to -1
        let mut pidfd = -1;
        let _ = clone.flag_pidfd(&mut pidfd);

        // We have a concern that the "clone" API changes/breaks in the future and this
        // test is intended to help safeguard against that!
        // We check that the clone we kept has set the first flag we set above.
        assert_eq!(clone.as_clone_args().flags, Flags::PIDFD.bits());

        // Freeze the parent until the child calls execvp
        let _ = clone.flag_vfork();

        // Manage SIGCHLD for the nested process
        // Define SIGCHLD for signal handler
        let _ = clone.exit_signal(SIGCHLD as u64);

        // [ Namespaces and Isolation ]

        let mut isolation = Isolation::new(name);
        isolation.setup(&iso_ctl)?;

        // Always unshare the Cgroup namespace
        let _ = clone.flag_newcgroup();

        // Isolate Network
        if iso_ctl.isolate_network {
            let _ = clone.flag_newnet();
        }

        // Isolate Process
        if iso_ctl.isolate_process {
            let _ = clone.flag_newpid();
            let _ = clone.flag_newns();
            let _ = clone.flag_newipc();
            let _ = clone.flag_newuts();
        }

        // Execute the clone system call and create the new process with the relevant namespaces.
        match unsafe { clone.call() }? {
            0 => {
                // child
                let command = {
                    unsafe {
                        command.pre_exec(move || {
                            isolation.isolate_process(&iso_ctl)?;
                            isolation.isolate_network(&iso_ctl)?;
                            Ok(())
                        })
                    }
                };

                let e = command.exec();
                error!("Unexpected exit from child command: {e:#?}");
                Err(e)
            }
            pid => {
                // parent
                info!("Nested auraed running with host pid {}", pid.clone());
                let process = procfs::process::Process::new(pid)
                    .map_err(io::Error::other)?;

                Ok(Self { process, pidfd, iso_ctl, client_socket })
            }
        }
    }

    /// Sends a graceful shutdown signal to the nested process.
    pub fn shutdown(&mut self) -> io::Result<ExitStatus> {
        // TODO: Here, SIGTERM works when using auraescript, but hangs(?) during unit tests.
        //       SIGKILL, however, works. The hang is avoided if the process is not isolated.
        //       Tests have not been done to figure out which namespace is the cause of the hang.
        self.do_kill(Some(SIGTERM))?;
        self.wait()
    }

    /// Sends a [SIGKILL] signal to the nested process.
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
        nix::sys::signal::kill(pid, signal)?;
        Ok(())
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        let pid = Pid::from_raw(self.process.pid);

        let status = loop {
            match waitpid(pid, None) {
                Ok(status) => break status,
                Err(Errno::EINTR) => continue,
                Err(e) => return Err(e.into()),
            }
        };

        let exit_status = match status {
            WaitStatus::Exited(_, code) => {
                trace!("Pid {pid} exited with code {code}");
                ExitStatus::from_raw(code << 8)
            }
            WaitStatus::Signaled(_, sig, core_dumped) => {
                if core_dumped {
                    error!("Pid {pid} killed by signal {sig} (core dumped)");
                } else {
                    trace!("Pid {pid} killed by signal {sig}");
                }
                ExitStatus::from_raw(sig as i32)
            }
            WaitStatus::Stopped(_, sig) => {
                error!("Pid {pid} unexpectedly stopped by signal {sig}");
                return Err(io::Error::other(format!(
                    "process {pid} stopped by signal {sig}"
                )));
            }
            WaitStatus::Continued(_) => {
                error!("Pid {pid} unexpectedly continued");
                return Err(io::Error::other(format!(
                    "process {pid} continued unexpectedly"
                )));
            }
            WaitStatus::PtraceEvent(_, sig, event) => {
                error!(
                    "Pid {pid} unexpected ptrace event {event} (signal {sig})"
                );
                return Err(io::Error::other(format!(
                    "unexpected ptrace event for process {pid}"
                )));
            }
            WaitStatus::PtraceSyscall(_) => {
                error!("Pid {pid} unexpected ptrace syscall-stop");
                return Err(io::Error::other(format!(
                    "unexpected ptrace syscall-stop for process {pid}"
                )));
            }
            WaitStatus::StillAlive => {
                error!("Pid {pid} still alive after waitpid");
                return Err(io::Error::other(format!(
                    "process {pid} still alive after waitpid"
                )));
            }
        };

        Ok(exit_status)
    }

    pub fn pid(&self) -> Pid {
        Pid::from_raw(self.process.pid)
    }
}
