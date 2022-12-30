use super::process::Process;
use super::{ExecutableName, ExecutableSpec};
use crate::logging::log_channel::LogChannel;
use nix::sys::signal::SIGKILL;
use nix::unistd::Pid;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::{
    io,
    process::{Command, ExitStatus, Stdio},
};
use tracing::{info, info_span};

#[derive(Debug)]
struct ExecutableInner {
    name: ExecutableName,
    description: String,
    state: ExecutableState,
    stdout: LogChannel,
    stderr: LogChannel,
}

#[derive(Debug)]
pub struct Executable {
    // TODO: consider RWLock?
    inner: Arc<Mutex<ExecutableInner>>,
    log_thread: Option<std::thread::JoinHandle<io::Result<()>>>,
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
        process: Arc<Mutex<Process>>,
    },
    Stopped(ExitStatus),
}

impl Executable {
    pub fn new<T: Into<ExecutableSpec>>(spec: T) -> Self {
        let ExecutableSpec { name, description, command } = spec.into();
        let state = ExecutableState::Init { command };
        let inner = Arc::new(Mutex::new(ExecutableInner {
            name: name.clone(),
            description,
            state,
            stdout: LogChannel::new(format!("{name}::stdout")),
            stderr: LogChannel::new(format!("{name}::stderr")),
        }));
        Self { inner: inner.clone(), log_thread: Some(spawn_log_thread(inner)) }
    }

    pub fn name(&self) -> io::Result<ExecutableName> {
        let inner = self
            .inner
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(inner.name.clone())
    }

    pub fn description(&self) -> io::Result<String> {
        let inner = self
            .inner
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(inner.description.clone())
    }

    /// Starts the underlying process.
    /// Does nothing if [Executable] has previously been started.
    pub fn start(&mut self) -> io::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let ExecutableState::Init { command } = &mut inner.state else {
            return Ok(());
        };

        let process = exec(command)?;

        inner.state = ExecutableState::Started {
            program: command.get_program().to_os_string(),
            args: command.get_args().map(|arg| arg.to_os_string()).collect(),
            process: Arc::new(Mutex::new(process)),
        };

        Ok(())
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub fn kill(&mut self) -> io::Result<Option<ExitStatus>> {
        let exit_status: Option<ExitStatus>;
        {
            let mut inner = self.inner.lock().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, e.to_string())
            })?;
            match &mut inner.state {
                ExecutableState::Init { .. } => exit_status = None,
                ExecutableState::Started { process, .. } => {
                    let proc_status: ExitStatus;
                    {
                        let mut proc = process.lock().map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e.to_string())
                        })?;
                        proc.kill(Some(SIGKILL))?;
                        proc_status = proc.wait()?;
                    }
                    inner.state = ExecutableState::Stopped(proc_status);
                    exit_status = Some(proc_status);
                }
                ExecutableState::Stopped(status) => exit_status = Some(*status),
            };
        }
        self.log_thread
            .take()
            .map(std::thread::JoinHandle::join)
            .expect("thread panicked")
            .expect("join log_thread")
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(exit_status)
    }

    /// Returns the [Pid] while [Executable] is running, otherwise returns [None].
    pub fn pid(&self) -> io::Result<Option<Pid>> {
        let inner = self
            .inner
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let ExecutableState::Started { process, .. } = &inner.state else {
            return Ok(None);
        };
        let proc = process
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(Some(proc.pid()))
    }
}

/// Spawns a thread that produces log lines while the [Executable] is running.
fn spawn_log_thread(
    inner: Arc<Mutex<ExecutableInner>>,
) -> std::thread::JoinHandle<io::Result<()>> {
    let local_inner = inner;
    std::thread::spawn(move || -> io::Result<()> {
        let mut running = true;
        while running {
            let inner = local_inner.lock().map_err(|e| {
                io::Error::new(io::ErrorKind::Other, e.to_string())
            })?;
            let _span =
                info_span!("running process", name = ?inner.name).entered();
            match &inner.state {
                ExecutableState::Init { .. } => {}
                ExecutableState::Started { process, .. } => {
                    let mut proc = process.lock().map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e.to_string())
                    })?;
                    let lines = proc.read_stdout()?;
                    for line in lines {
                        info!(
                            level = "info",
                            channel = inner.stdout.name(),
                            line
                        );
                        LogChannel::log_line(
                            inner.stdout.get_producer().clone(),
                            line.to_string(),
                        );
                    }
                    let lines = proc.read_stderr()?;
                    for line in lines {
                        info!(
                            level = "error",
                            channel = inner.stderr.name(),
                            line
                        );
                        LogChannel::log_line(
                            inner.stderr.get_producer().clone(),
                            line.to_string(),
                        );
                    }
                }
                ExecutableState::Stopped { .. } => running = false,
            }
        }
        Ok(())
    })
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
