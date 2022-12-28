use tracing::{info, Level};
use tracing_subscriber::{prelude::*, util::TryInitError};

const AURAED_SYSLOG_IDENT: &str = "auraed";

#[derive(thiserror::Error, Debug)]
pub(crate) enum LoggingError {
    #[error("Failed to setup basic tracing: {source:?}")]
    SetupFailure { source: Box<dyn std::error::Error> },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    TryInitError(#[from] TryInitError),
}

pub(crate) fn init(verbose: bool) -> Result<(), LoggingError> {
    // The logger will log to stdout and possibly jourald depending on
    // whether we are initializing as pid1 or not.
    // We hold the opinion that the program is either "verbose"
    // or it's not.
    //
    // Normal mode: Info, Warn, Error
    // Verbose mode: Debug, Trace, Info, Warn, Error
    let tracing_level = if verbose { Level::TRACE } else { Level::INFO };

    match std::process::id() {
        1 => init_pid1_logging(tracing_level),
        _ => init_journald_logging(tracing_level),
    }
}

fn init_journald_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing syslog logging");
    let journald_layer = tracing_journald::layer()?
        .with_syslog_identifier(AURAED_SYSLOG_IDENT.into());

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .finish()
        .with(journald_layer)
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
