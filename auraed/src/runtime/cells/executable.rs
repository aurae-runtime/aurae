use crate::runtime::cells::ExecutableName;
use cgroups_rs::CgroupPid;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus};
use tracing::info;

#[derive(Debug)]
pub(crate) struct Executable {
    pub name: ExecutableName,
    pub command: String,
    pub description: String,
    state: ExecutableState,
}

#[derive(Debug)]
enum ExecutableState {
    Init,
    Started(Child),
    Stopped(ExitStatus),
}

impl Executable {
    pub fn new(
        name: ExecutableName,
        command: String,
        description: String,
    ) -> Self {
        Self { name, command, description, state: ExecutableState::Init }
    }

    /// Starts the executable and returns the pid.
    /// If the executable is already started, just returns the pid.
    ///
    /// # Arguments
    ///
    /// * `pre_exec` - Run 'pre_exec' hooks from the context of the soon-to-be launched child.
    pub fn start<F>(
        &mut self,
        pre_exec: Option<F>,
    ) -> Result<CgroupPid, io::Error>
    where
        F: FnMut() -> io::Result<()> + Send + Sync + 'static,
    {
        match &self.state {
            ExecutableState::Started(child) => Ok((child.id() as u64).into()),

            ExecutableState::Init | ExecutableState::Stopped(_) => {
                let mut command = Command::new("/usr/bin/sh");
                let mut command = command.args(["-c", &self.command]);

                if let Some(pre_exec) = pre_exec {
                    command = unsafe { command.pre_exec(pre_exec) };
                }

                let child = command.spawn()?;
                let pid = (child.id() as u64).into();
                self.state = ExecutableState::Started(child);

                Ok(pid)
            }
        }
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub fn kill(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        match &mut self.state {
            ExecutableState::Init => Ok(None),
            ExecutableState::Started(child) => {
                let id = child.id();
                child.kill()?;
                let exit_status = child.wait()?;

                info!(
                    "Executable with pid {id} exited with status {exit_status}",
                );

                self.state = ExecutableState::Stopped(exit_status);
                Ok(Some(exit_status))
            }
            ExecutableState::Stopped(exit_status) => Ok(Some(*exit_status)),
        }
    }

    /// Returns the [CgroupPid] while [Executable] is running, otherwise returns [None].
    pub fn pid(&self) -> Option<CgroupPid> {
        match &self.state {
            ExecutableState::Started(child) => Some((child.id() as u64).into()),
            ExecutableState::Init | ExecutableState::Stopped(_) => None,
        }
    }
}
