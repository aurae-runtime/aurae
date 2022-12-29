use super::process::Process;
use super::{ExecutableName, ExecutableSpec};
use crate::logging::log_channel::LogChannel;
use nix::sys::signal::SIGKILL;
use nix::unistd::Pid;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufRead;
use std::os::unix::prelude::FromRawFd;
use std::sync::Arc;
use std::{
    io,
    process::{Command, ExitStatus, Stdio},
};
use tracing::{event, warn, Level};

#[derive(Debug)]
pub struct Executable {
    pub name: ExecutableName,
    pub description: String,
    state: ExecutableState,
    stdout: Arc<LogChannel>,
    stderr: Arc<LogChannel>,
    // TODO: log_thread: Option<std::thread::JoinHandle<io::Result<()>>>,
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
        let stdout = Arc::new(LogChannel::new(format!("{}::stdout", name)));
        let stderr = Arc::new(LogChannel::new(format!("{}::stderr", name)));
        Self {
            name,
            description,
            state: ExecutableState::Init { command },
            stdout,
            stderr,
            // TODO: log_thread: None,
        }
    }

    /// Starts the underlying process.
    /// Does nothing if [Executable] has previously been started.
    pub fn start(&mut self) -> io::Result<()> {
        let ExecutableState::Init { command } = &mut self.state else {
            return Ok(());
        };

        let process = exec(command)?;

        // TODO: self.log_thread = Some(self.spawn_log_thread());

        self.state = ExecutableState::Started {
            program: command.get_program().to_os_string(),
            args: command.get_args().map(|arg| arg.to_os_string()).collect(),
            process,
        };

        Ok(())
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub fn kill(&mut self) -> io::Result<Option<ExitStatus>> {
        match &mut self.state {
            ExecutableState::Init { .. } => Ok(None),
            ExecutableState::Started { process, .. } => {
                process.kill(Some(SIGKILL))?;
                let exit_status = process.wait()?;
                // TODO
                //self.log_thread
                //    .unwrap()
                //    .join()
                //    .expect("logging thread panicked")?;
                for line in self.read_stdout()? {
                    event!(
                        Level::INFO,
                        level = "info",
                        channel = self.stdout.name(),
                        line
                    );
                    LogChannel::log_line(
                        self.stdout.get_producer().clone(),
                        line.to_string(),
                    );
                }
                for line in self.read_stderr()? {
                    event!(
                        Level::INFO,
                        level = "error",
                        channel = self.stderr.name(),
                        line
                    );
                    LogChannel::log_line(
                        self.stderr.get_producer().clone(),
                        line.to_string(),
                    );
                }
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

    /// Returns any unread lines from stdout if [Executable] is running, otherwise returns an empty
    /// [Vec].
    pub fn read_stdout(&mut self) -> io::Result<Vec<String>> {
        let ExecutableState::Started { process, .. } = &mut self.state else {
            warn!("attempted to read stdout on process that was not started");
            return Ok(vec![]);
        };

        let mut output = Vec::new();
        match process {
            Process::Spawned(child) => {
                if let Some(stdout) = child.stdout.as_mut() {
                    for line in io::BufReader::new(stdout).lines() {
                        output.push(line?);
                    }
                };
            }
            Process::Cloned { process, .. } => {
                let fd = process
                    .fd_from_fd(1)
                    .map_err(|e| {
                        io::Error::new(io::ErrorKind::BrokenPipe, e.to_string())
                    })?
                    .fd;
                let f = unsafe { File::from_raw_fd(fd) };

                for line in io::BufReader::new(f).lines() {
                    output.push(line?);
                }
            }
        };
        Ok(output)
    }

    /// Returns any unread lines from stderr if [Executable] is running, otherwise returns an empty
    /// [Vec].
    pub fn read_stderr(&mut self) -> io::Result<Vec<String>> {
        let ExecutableState::Started { process, .. } = &mut self.state else {
            return Ok(vec![]);
        };

        let mut output = Vec::new();
        match process {
            Process::Spawned(child) => {
                if let Some(stdout) = child.stderr.as_mut() {
                    for line in io::BufReader::new(stdout).lines() {
                        output.push(line?);
                    }
                };
            }
            Process::Cloned { process, .. } => {
                let fd = process
                    .fd_from_fd(2)
                    .map_err(|e| {
                        io::Error::new(io::ErrorKind::BrokenPipe, e.to_string())
                    })?
                    .fd;
                let f = unsafe { File::from_raw_fd(fd) };
                for line in io::BufReader::new(f).lines() {
                    output.push(line?);
                }
            }
        };
        Ok(output)
    }

    // TODO:
    // /// Spawns a thread that produces log lines while the [Executable] is running.
    // fn spawn_log_thread(&mut self) -> std::thread::JoinHandle<io::Result<()>> {
    //     let local_stdout = self.stdout.clone();
    //     let local_stderr = self.stderr.clone();
    //     std::thread::spawn(move || -> io::Result<()> {
    //         let mut running = true;
    //         while running {
    //             match self.state {
    //                 ExecutableState::Init { .. } => {}
    //                 ExecutableState::Started { .. } => {
    //                     // TODO: consider changing the pipes to use io::BufReader::new(file).lines()
    //                     // and iterating over those lines here.
    //                     if let Some(stdout) = self.read_stdout()? {
    //                         event!(
    //                             Level::INFO,
    //                             channel = local_stdout.name(),
    //                             stdout
    //                         );
    //                         // TODO:
    //                         // LogChannel::log_line(
    //                         //     self.stdout.get_producer().clone(),
    //                         //     stdout,
    //                         // );
    //                     };
    //                     if let Some(stderr) = self.read_stderr()? {
    //                         event!(
    //                             Level::ERROR,
    //                             channel = local_stderr.name(),
    //                             stderr
    //                         );
    //                         // TODO:
    //                         // LogChannel::log_line(
    //                         //     self.stderr.get_producer().clone(),
    //                         //     stderr,
    //                         // );
    //                     };
    //                 }
    //                 ExecutableState::Stopped { .. } => running = false,
    //             }
    //         }
    //         Ok(())
    //     })
    // }
}

// Start the child process
//
// Here is where we launch an executable within the context of a parent Cell.
// Aurae makes the assumption that all Executables within a cell share the
// same namespace isolation rules set up upon creation of the cell.
fn exec(command: &mut Command) -> io::Result<Process> {
    let child = command
        .current_dir("/")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(Process::new_from_spawn(child))
}
