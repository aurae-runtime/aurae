use super::process::Process;
use super::{ExecutableName, ExecutableSpec};
use nix::sys::signal::SIGKILL;
use nix::unistd::Pid;
use std::ffi::OsString;
use std::process::Command;
use std::{io, process::ExitStatus};

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

impl Executable {
    pub fn new<T: Into<ExecutableSpec>>(spec: T) -> Self {
        let ExecutableSpec { name, description, command } = spec.into();
        Self { name, description, state: ExecutableState::Init { command } }
    }

    /// Starts the underlying process.
    /// Does nothing if [Executable] has previously been started.
    pub fn start(&mut self) -> io::Result<()> {
        let ExecutableState::Init { command } = &mut self.state else {
            return Ok(());
        };

        let process = exec(command)?;

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
                process.kill(Some(SIGKILL))?;
                let exit_status = process.wait()?;
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

        Some(process.pid())
    }
}

// Start the child process
//
// Here is where we launch an executable within the context of a parent Cell.
// Aurae makes the assumption that all Executables within a cell share the
// same namespace isolation rules set up upon creation of the cell.
fn exec(command: &mut Command) -> io::Result<Process> {
    let child = command.current_dir("/").spawn()?;
    Ok(Process::new_from_spawn(child))
}
