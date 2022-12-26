use crate::{ExecutableName, ExecutableSpec, SharedNamespaces};
use nix::{
    libc::SIGCHLD,
    sys::{signal::SIGKILL, wait::WaitStatus},
    unistd::Pid,
};
use procfs::process::Process;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{
    io::{self, ErrorKind},
    os::unix::process::ExitStatusExt,
    process::ExitStatus,
};
use tracing::info;

#[derive(Debug)]
pub struct Executable {
    pub name: ExecutableName,
    pub description: String,
    state: ExecutableState,
}

#[derive(Debug)]
enum ExecutableState {
    Init {
        command: Command,
        role: ExecutableRole,
    },
    Started {
        #[allow(unused)]
        program: OsString,
        #[allow(unused)]
        args: Vec<OsString>,
        process: Process,
    },
    Stopped(ExitStatus),
}

#[derive(Debug)]
enum ExecutableRole {
    Pid1 { shared_namespaces: SharedNamespaces },
    Other,
}

impl Executable {
    pub fn new_pid1<T: Into<ExecutableSpec>>(
        spec: T,
        shared_namespaces: SharedNamespaces,
    ) -> Self {
        let ExecutableSpec { name, description, command } = spec.into();
        Self {
            name,
            description,
            state: ExecutableState::Init {
                command,
                role: ExecutableRole::Pid1 { shared_namespaces },
            },
        }
    }

    pub fn new<T: Into<ExecutableSpec>>(spec: T) -> Self {
        let ExecutableSpec { name, description, command } = spec.into();
        Self {
            name,
            description,
            state: ExecutableState::Init {
                command,
                role: ExecutableRole::Other,
            },
        }
    }

    /// Starts the underlying process.
    /// Does nothing if [Executable] has previously been started.
    pub fn start(&mut self) -> io::Result<()> {
        let ExecutableState::Init { command, role } = &mut self.state else {
            return Ok(());
        };

        let process = match role {
            ExecutableRole::Pid1 { shared_namespaces } => {
                exec_pid1(command, shared_namespaces)
            }
            ExecutableRole::Other => exec(command),
        }?;

        self.state = ExecutableState::Started {
            program: command.get_program().to_os_string(),
            args: command.get_args().map(|arg| arg.to_os_string()).collect(),
            process,
        };

        Ok(())
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub fn kill(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        match &mut self.state {
            ExecutableState::Init { .. } => Ok(None),
            ExecutableState::Started { process, .. } => {
                let pid = Pid::from_raw(process.pid);
                nix::sys::signal::kill(pid, SIGKILL)
                    .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

                // https://pubs.opengroup.org/onlinepubs/9699919799/functions/waitpid.html
                // The waitpid() function obtains status information for process termination,
                // and optionally process stop and/or continue, from a specified subset of the child processes.
                // If pid is greater than 0, it specifies the process ID of a single child process for which status is requested.
                let WaitStatus::Exited(_, exit_status) = nix::sys::wait::waitpid(pid, None)
                    .map_err(|e| io::Error::from_raw_os_error(e as i32))? else {
                    unreachable!("we specify a pid > 0, with no flags, so should only return on termination")
                };

                let exit_status = ExitStatus::from_raw(exit_status);

                info!(
                    "Executable with pid {pid} exited with status {exit_status}",
                );

                self.state = ExecutableState::Stopped(exit_status);
                Ok(Some(exit_status))
            }
            ExecutableState::Stopped(exit_status) => Ok(Some(*exit_status)),
        }
    }

    /// Returns the [Pid] while [Executable] is running, otherwise returns [None].
    pub fn pid(&self) -> Option<Pid> {
        let ExecutableState::Started { process, .. } = &self.state else {
            return None;
        };

        Some(Pid::from_raw(process.pid))
    }
}

fn exec_pid1(
    command: &mut Command,
    shared_namespaces: &SharedNamespaces,
) -> io::Result<Process> {
    // Clone docs: https://man7.org/linux/man-pages/man2/clone.2.html

    // If this signal is specified as anything other than SIGCHLD, then the
    //        parent process must specify the __WALL or __WCLONE options when
    //        waiting for the child with wait(2).  If no signal (i.e., zero) is
    //        specified, then the parent process is not signaled when the child
    //        terminates.
    let signal = SIGCHLD;

    let mut pidfd = -1;
    let mut clone = clone3::Clone3::default();
    clone.flag_pidfd(&mut pidfd);
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

    let pid = unsafe { clone.call() }
        .map_err(|e| io::Error::from_raw_os_error(e.0))?;

    if pid == 0 {
        // we are in the child

        let command = command.current_dir("/");

        let command = {
            let shared_namespaces = shared_namespaces.clone();
            unsafe {
                command.pre_exec(move || {
                    if !shared_namespaces.mount {
                        // We can potentially do some setup here, and then leave the rest
                        // to auraed's init. We would need flags to signal it is a nested PID1

                        // remount as private
                        nix::mount::mount(
                            Some("."),
                            ".",
                            None::<&str>,
                            nix::mount::MsFlags::MS_SLAVE
                                | nix::mount::MsFlags::MS_REC,
                            Some(""),
                        )
                        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

                        nix::mount::mount(
                            Some("proc"),
                            "/proc",
                            Some("proc"),
                            nix::mount::MsFlags::empty(),
                            None::<&[u8]>,
                        )
                        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;
                    }

                    Ok(())
                })
            }
        };

        command.exec();
    }

    // we are in the parent again

    let process =
        Process::new(pid).map_err(|e| io::Error::new(ErrorKind::Other, e))?;

    Ok(process)
}

// Start the child process
//
// Here is where we launch an executable within the context of a parent Cell.
// Aurae makes the assumption that all Executables within a cell share the
// same namespace isolation rules set up upon creation of the cell.
fn exec(command: &mut Command) -> io::Result<Process> {
    let child = command.current_dir("/").spawn()?;

    let process = Process::new(child.id() as i32)
        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

    Ok(process)
}
