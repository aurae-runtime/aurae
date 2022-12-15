use cgroups_rs::CgroupPid;
use log::info;
use std::io;
use std::process::{Child, Command, ExitStatus};

#[derive(Debug)]
pub(crate) struct Executable(Child);

impl Executable {
    pub fn start(mut command: Command) -> io::Result<Self> {
        let child = command.spawn()?;
        Ok(Self(child))
    }

    pub fn kill(&mut self) -> io::Result<ExitStatus> {
        let id = self.0.id();
        self.0.kill()?;
        let exit_status = self.0.wait()?;
        info!("Executable with pid {id} exited with status {exit_status}");
        Ok(exit_status)
    }

    pub fn pid(&self) -> CgroupPid {
        (self.0.id() as u64).into()
    }
}
