use tracing::Level;

#[derive(thiserror::Error, Debug)]
pub(crate) enum LoggingError {
    #[error("Failed to setup tracing: {source:?}")]
    TracingSetupFailure { source: Box<dyn std::error::Error> },
}

pub(crate) fn init(logger_level: Level) -> Result<(), LoggingError> {
    match std::process::id() {
        1 => init_pid1_logging(logger_level),
        _ => init_syslog_logging(logger_level),
    }
}

fn init_syslog_logging(tracing_level: Level) -> Result<(), LoggingError> {
    // TODO: multiple subscribers
    // TODO: journald
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .try_init()
        .map_err(|e| LoggingError::TracingSetupFailure { source: e })
}

// To discuss here https://github.com/aurae-runtime/auraed/issues/24:
//      The "syslog" logger requires unix sockets.
//      Syslog assumes that either /dev/log or /var/run/syslog are available [1].
//      We need to discuss if there is a use case to log via unix sockets,
//      other than fullfill the requirement of syslog crate.
//      For now, auraed distinguishes between pid1 system and local (dev environment) logging.
//      [1] https://docs.rs/syslog/latest/src/syslog/lib.rs.html#232-243
fn init_pid1_logging(tracing_level: Level) -> Result<(), LoggingError> {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .try_init()
        .map_err(|e| LoggingError::TracingSetupFailure { source: e })
}
