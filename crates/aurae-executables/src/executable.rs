use crate::{ExecutableName, ExecutableSpec, SharedNamespaces};
use nix::{
    libc,
    libc::SIGCHLD,
    sched::CloneFlags,
    sys::{signal::SIGKILL, wait::WaitStatus},
    unistd::Pid,
};
use procfs::process::Process;
use std::ffi::CStr;
use std::{
    ffi::CString,
    io::{self, ErrorKind},
    os::unix::process::ExitStatusExt,
    process::ExitStatus,
};
use tracing::info;

#[derive(Debug)]
pub struct Executable {
    pub name: ExecutableName,
    pub command: CString,
    pub description: String,
    state: ExecutableState,
}

#[derive(Debug)]
enum ExecutableState {
    Init(ExecutableRole),
    Started(Process),
    Stopped(ExitStatus),
}

#[derive(Debug)]
pub enum ExecutableRole {
    // #[cfg(target_os = "linux")]
    Pid1 { shared_namespaces: SharedNamespaces },
    Other,
}

impl Executable {
    pub fn new_pid1<T: Into<ExecutableSpec>>(
        spec: T,
        shared_namespaces: SharedNamespaces,
    ) -> Self {
        let ExecutableSpec { name, command, description } = spec.into();
        Self {
            name,
            command,
            description,
            state: ExecutableState::Init(ExecutableRole::Pid1 {
                shared_namespaces,
            }),
        }
    }

    pub fn new<T: Into<ExecutableSpec>>(spec: T) -> Self {
        let ExecutableSpec { name, command, description } = spec.into();
        Self {
            name,
            command,
            description,
            state: ExecutableState::Init(ExecutableRole::Other),
        }
    }

    /// Starts the executable and returns the pid.
    /// If the executable is running, returns the pid.
    /// If the executable has stopped, returns [None]
    pub fn start(&mut self) -> io::Result<Option<Pid>> {
        if let ExecutableState::Started(process) = &self.state {
            return Ok(Some(Pid::from_raw(process.pid)));
        }

        let ExecutableState::Init(role) = &self.state else {
            return Ok(None);
        };

        let process = match role {
            ExecutableRole::Pid1 { shared_namespaces } => {
                exec_pid1(self.command.clone(), shared_namespaces)
            }
            ExecutableRole::Other => exec(self.command.clone()),
        }?;

        let pid = Pid::from_raw(process.pid);
        self.state = ExecutableState::Started(process);

        Ok(Some(pid))
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub fn kill(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        match &mut self.state {
            ExecutableState::Init(_) => Ok(None),
            ExecutableState::Started(process) => {
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
        match &self.state {
            ExecutableState::Started(process) => {
                Some(Pid::from_raw(process.pid))
            }
            ExecutableState::Init(_) | ExecutableState::Stopped(_) => None,
        }
    }
}

fn exec_pid1(
    _command: CString,
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
        .map_err(|e| io::Error::from_raw_os_error(e.0 as i32))?;

    if pid == 0 {
        // we are in the child
        nix::mount::mount(
            Some("proc"),
            "/proc",
            Some("proc"),
            nix::mount::MsFlags::empty(),
            None::<&[u8]>,
        )
        .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        let args: Vec<&CStr> = vec![];
        nix::unistd::execvp(&CString::new("aurae-init").expect("valid"), &args)
            .map_err(|e| io::Error::from_raw_os_error(e as i32))?;

        unsafe {
            libc::exit(1);
        }
    }

    // we are in the parent again

    let process =
        Process::new(pid).map_err(|e| io::Error::new(ErrorKind::Other, e))?;

    Ok(process)
}

fn exec(_command: CString) -> io::Result<Process> {
    todo!()
}
