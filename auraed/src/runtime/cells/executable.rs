use crate::runtime::cells::ExecutableName;
use cgroups_rs::CgroupPid;
use log::info;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus};
use thiserror::Error;

type Result<T> = std::result::Result<T, ExecutableError>;

#[derive(Debug)]
pub(crate) struct Executable {
    name: ExecutableName,
    child: Child,
}

impl Executable {
    pub fn start(
        name: ExecutableName,
        mut command: Command,
        args: Vec<String>,
        _description: String,
    ) -> Result<Self> {
        // Currently takes and returns &mut Command. Calling it as `command.args(args)` leaves us
        // vulnerable to the implementation changing to taking ownership and returning a new command,
        // which we then ignore with `let _ =`. To prevent that vulnerability, ensure `.args` takes
        // command as a &mut, so we always retain ownership, or the compiler errors.
        // This is done (instead of just reassigning) so that the command can be passed into the error.
        #[allow(clippy::needless_borrow)]
        let _ = (&mut command).args(args);

        // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
        let _ = {
            let name_clone = name.clone();
            unsafe {
                #[allow(clippy::needless_borrow)]
                (&mut command)
                    .pre_exec(move || aurae_process_pre_exec(&name_clone))
            }
        };

        match command.spawn() {
            Ok(child) => Ok(Self { name, child }),
            Err(e) => Err(ExecutableError::FailedToStart {
                executable_name: name,
                command,
                source: e,
            }),
        }
    }

    pub fn kill(&mut self) -> Result<ExitStatus> {
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

fn aurae_process_pre_exec(exe_name: &ExecutableName) -> io::Result<()> {
    info!("CellService: aurae_process_pre_exec(): {exe_name}");
    // Here we are executing as the new spawned pid.
    // This is a place where we can "hook" into all processes
    // started with Aurae in the future. Similar to kprobe/uprobe
    // in Linux or LD_PRELOAD in libc.
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::process::Command;

    #[test]
    fn test_command_implementation() {
        let in_args: Vec<OsString> =
            vec!["hi".into(), "from".into(), "test".into()];

        let mut command = Command::new("echo");
        let _ = (&mut command).args(in_args.clone());
        let out_args: Vec<_> =
            command.get_args().into_iter().map(|x| x.to_os_string()).collect();
        assert_eq!(in_args, out_args);
    }
}
