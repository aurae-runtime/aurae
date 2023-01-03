use super::{ExecutableName, ExecutableSpec};
use crate::logging::log_channel::LogChannel;
use nix::unistd::Pid;
use std::{
    ffi::OsString,
    io,
    process::{ExitStatus, Stdio},
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tracing::{info, info_span};

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
        child: Child,
        stdout: JoinHandle<()>,
        stderr: JoinHandle<()>,
    },
    Stopped(ExitStatus),
}

impl Executable {
    pub fn new<T: Into<ExecutableSpec>>(spec: T) -> Self {
        let ExecutableSpec { name, description, command } = spec.into();
        let state = ExecutableState::Init { command };
        Self { name, description, state }
    }

    /// Starts the underlying process.
    /// Does nothing if [Executable] has previously been started.
    pub fn start(&mut self) -> io::Result<()> {
        let ExecutableState::Init { command } = &mut self.state else {
            return Ok(());
        };

        let mut child = command
            .current_dir("/")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout");
        let log_channel = LogChannel::new(format!("{}::stdout", self.name));
        let span = info_span!("running process", name = ?self.name);
        let stdout = tokio::spawn(async move {
            let log_channel = log_channel;
            let mut span = Some(span);
            let mut stdout = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = stdout.next_line().await {
                let entered_span = span.take().expect("span").entered();
                info!(level = "info", channel = log_channel.name, line);
                log_channel.send(line);
                span = Some(entered_span.exit());
            }
        });

        let stderr = child.stderr.take().expect("stderr");
        let log_channel = LogChannel::new(format!("{}::stderr", self.name));
        let span = info_span!("running process", name = ?self.name);
        let stderr = tokio::spawn(async move {
            let log_channel = log_channel;
            let mut span = Some(span);
            let mut stderr = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = stderr.next_line().await {
                let entered_span = span.take().expect("span").entered();
                info!(level = "error", channel = log_channel.name, line);
                log_channel.send(line);
                span = Some(entered_span.exit());
            }
        });

        self.state = ExecutableState::Started {
            program: command.as_std().get_program().to_os_string(),
            args: command
                .as_std()
                .get_args()
                .map(|arg| arg.to_os_string())
                .collect(),
            child,
            stdout,
            stderr,
        };

        Ok(())
    }

    /// Stops the executable and returns the [ExitStatus].
    /// If the executable has never been started, returns [None].
    pub async fn kill(&mut self) -> io::Result<Option<ExitStatus>> {
        Ok(match &mut self.state {
            ExecutableState::Init { .. } => None,
            ExecutableState::Started { child, stdout, stderr, .. } => {
                child.kill().await?;
                let exit_status = child.wait().await?;
                let _ = tokio::join!(stdout, stderr);
                self.state = ExecutableState::Stopped(exit_status);
                Some(exit_status)
            }
            ExecutableState::Stopped(status) => Some(*status),
        })
    }

    /// Returns the [Pid] while [Executable] is running, otherwise returns [None].
    pub fn pid(&self) -> io::Result<Option<Pid>> {
        let ExecutableState::Started { child: process, .. } = &self.state else {
            return Ok(None);
        };

        Ok(process.id().map(|id| Pid::from_raw(id as i32)))
    }
}
