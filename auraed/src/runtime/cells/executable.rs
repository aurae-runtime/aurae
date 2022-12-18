use crate::runtime::cells::ExecutableName;
use cgroups_rs::CgroupPid;
use log::info;
use std::io;
use unshare::Child;
use unshare::Command;
use unshare::Error;
use unshare::ExitStatus;

#[derive(Debug)]
pub(crate) struct Executable {
    pub name: ExecutableName,
    pub command: String,
    pub args: Vec<String>,
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
        args: Vec<String>,
        description: String,
    ) -> Self {
        Self { name, command, args, description, state: ExecutableState::Init }
    }

    /// Starts the executable and returns the pid.
    /// If the executable is already started, just returns the pid.
    pub fn start(&mut self) -> Result<CgroupPid, Error> {
        match &self.state {
            ExecutableState::Started(child) => Ok((child.id() as u64).into()),

            ExecutableState::Init | ExecutableState::Stopped(_) => {
                let mut command = Command::new(&self.command);
                let command = command.args(&self.args);

                // Run 'pre_exec' hooks from the context of the soon-to-be launched child.

                let command = {
                    let executable_name = self.name.clone();
                    unsafe {
                        command.pre_exec(move || {
                            aurae_process_pre_exec(&executable_name)
                        })
                    }
                };

                let child = command.spawn()?;
                let pid = (child.id() as u64).into();
                self.state = ExecutableState::Started(child);

                Ok(pid)
            }
        }
    }

    /// Stops the executable and returns the `ExitStatus`.
    /// If the executable has never been started, returns `None`.
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

    /// Returns the `pid` while `Executable` is running, otherwise returns `None`.
    pub fn pid(&self) -> Option<CgroupPid> {
        match &self.state {
            ExecutableState::Started(child) => Some((child.id() as u64).into()),
            ExecutableState::Init | ExecutableState::Stopped(_) => None,
        }
    }
}

fn aurae_process_pre_exec(executable_name: &ExecutableName) -> io::Result<()> {
    info!("CellService: aurae_process_pre_exec(): {executable_name}");
    // Here we are executing as the new spawned pid.
    // This is a place where we can "hook" into all processes
    // started with Aurae in the future. Similar to kprobe/uprobe
    // in Linux or LD_PRELOAD in libc.
    Ok(())
}
