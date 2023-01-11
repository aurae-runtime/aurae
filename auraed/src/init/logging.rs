use tracing::{info, Level};
use tracing_rfc_5424::{
    rfc3164::Rfc3164, tracing::TrivialTracingFormatter, transport::UnixSocket,
};
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[derive(thiserror::Error, Debug)]
pub(crate) enum LoggingError {
    #[error("Failed to setup basic tracing: {source:?}")]
    SetupFailure { source: Box<dyn std::error::Error> },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),

    #[error(transparent)]
    SyslogError(#[from] tracing_rfc_5424::layer::Error),
}

pub(crate) fn init(verbose: bool, container: bool) -> Result<(), LoggingError> {
    // The logger will log to stdout.
    //
    // We hold the opinion that the program is either "verbose"
    // or it's not.
    //
    // Normal mode: Info, Warn, Error
    // Verbose mode: Debug, Trace, Info, Warn, Error
    let tracing_level = if verbose { Level::TRACE } else { Level::INFO };

    if container {
        init_container_logging(tracing_level)
    } else {
        match std::process::id() {
            1 => init_pid1_logging(tracing_level),
            _ => init_daemon_logging(tracing_level),
        }
    }
}

fn init_container_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing container logging");

    // Stdout
    let stdout_layer = tracing_subscriber::Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(stdout_layer)
        .try_init()
        .map_err(|e| e.into())
}

/// when we run as a daemon we want to log to stdout and syslog.
fn init_daemon_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing syslog logging");

    // Syslog
    let syslog_layer = tracing_rfc_5424::layer::Layer::<
        tracing_subscriber::Registry,
        Rfc3164,
        TrivialTracingFormatter,
        UnixSocket,
    >::try_default()?;

    // Stdout
    let stdout_layer = tracing_subscriber::Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(syslog_layer)
        .with(stdout_layer)
        .try_init()
        .map_err(|e| e.into())
}

fn init_stdout_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing stdout logging");
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .finish()
        .try_init()
        .map_err(|e| e.into())
}

// To discuss here https://github.com/aurae-runtime/auraed/issues/24:
//      The "syslog" logger requires unix sockets.
//      Syslog assumes that either /dev/log or /var/run/syslog are available [1].
// TODO: We need to discuss if there is a use case to log via unix sockets,
//      other than fullfill the requirement of syslog crate.
//      [1] https://docs.rs/syslog/latest/src/syslog/lib.rs.html#232-243
fn init_pid1_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing pid1 logging");
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .try_init()
        .map_err(|e| LoggingError::SetupFailure { source: e })
}
