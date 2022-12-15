use crate::runtime::cells::ExecutableName;
use cgroups_rs::CgroupPid;
use log::info;
use std::io;
use std::process::{Child, Command, ExitStatus};
use thiserror::Error;

// TODO: I don't know how I (future-highway) feel about this now that I've spun out
//   `ExecutableError` from `CellsError`. It allows for consuming `Command` on `start`
//   (slight safety gain) but probably at the expense of mapping our errors to `Status`.
//   If there are no security concerns with exposing the underlying errors, then the dev overhead
//   of mapping to `Status` is no much, but I'm thinking under the assumption that we won't expose.
#[derive(Error, Debug)]
pub(crate) enum ExecutableError {
    #[error("failed to start executable '{executable_name}' ({command:?}) due to: {source}")]
    FailedToStart {
        executable_name: ExecutableName,
        command: Command,
        source: io::Error,
    },
    #[error("failed to stop executable '{executable_name}' ({executable_pid:?}) due to: {source}")]
    FailedToStopExecutable {
        executable_name: ExecutableName,
        executable_pid: CgroupPid,
        source: io::Error,
    },
}

#[derive(Debug)]
pub(crate) struct Executable {
    name: ExecutableName,
    child: Child,
}

impl Executable {
    pub fn start(
        name: ExecutableName,
        mut command: Command,
    ) -> Result<Self, ExecutableError> {
        match command.spawn() {
            Ok(child) => Ok(Self { name, child }),
            Err(e) => Err(ExecutableError::FailedToStart {
                executable_name: name,
                command,
                source: e,
            }),
        }
    }

    pub fn kill(&mut self) -> Result<ExitStatus, ExecutableError> {
        let id = self.child.id();

        self.child.kill().map_err(|e| {
            ExecutableError::FailedToStopExecutable {
                executable_name: self.name.clone(),
                executable_pid: self.pid(),
                source: e,
            }
        })?;

        let exit_status = self.child.wait().map_err(|e| {
            ExecutableError::FailedToStopExecutable {
                executable_name: self.name.clone(),
                executable_pid: self.pid(),
                source: e,
            }
        })?;

        info!("Executable with pid {id} exited with status {exit_status}");
        Ok(exit_status)
    }

    pub fn pid(&self) -> CgroupPid {
        (self.child.id() as u64).into()
    }
}
