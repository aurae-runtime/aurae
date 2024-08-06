/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */
use std::ffi::CStr;
use tracing::{info, Level};
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

#[derive(thiserror::Error, Debug)]
pub(crate) enum LoggingError {
    #[error("Failed to setup basic tracing: {source:?}")]
    SetupFailure { source: Box<dyn std::error::Error> },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    TryInitError(#[from] tracing_subscriber::util::TryInitError),

    #[error("Failed to setup syslog logging")]
    SyslogError,
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
    let stdout_layer = Layer::with_filter(
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
    let syslog_identity =
        CStr::from_bytes_with_nul(b"auraed\0").expect("valid CStr");
    let syslog_facility = Default::default();
    let syslog_options = syslog_tracing::Options::LOG_PID;
    let Some(syslog) = syslog_tracing::Syslog::new(
        syslog_identity,
        syslog_options,
        syslog_facility,
    ) else {
        return Err(LoggingError::SyslogError);
    };

    let syslog_layer = tracing_subscriber::fmt::layer().with_writer(syslog);

    // Stdout
    let stdout_layer = Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(syslog_layer)
        .with(stdout_layer)
        .try_init()
        .map_err(|e| e.into())
}

#[allow(unused)]
fn init_stdout_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing stdout logging");
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .finish()
        .try_init()
        .map_err(|e| e.into())
}

fn init_pid1_logging(tracing_level: Level) -> Result<(), LoggingError> {
    info!("initializing pid1 logging");
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(format!("auraed={tracing_level}"))
        .try_init()
        .map_err(|e| LoggingError::SetupFailure { source: e })
}