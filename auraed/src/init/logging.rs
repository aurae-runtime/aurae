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
use crate::logging::log_channel::LogChannel;
use tracing::{Level, info};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
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

// The logger will log to stdout.
//
// We hold the opinion that the program is either "verbose"
// or it's not.
//
// Normal mode: Info, Warn, Error
// Verbose mode: Debug, Trace, Info, Warn, Error
pub(crate) fn init(
    verbose: bool,
    container: bool,
    log_channel: LogChannel,
) -> Result<(), LoggingError> {
    let tracing_level = if verbose { Level::TRACE } else { Level::INFO };

    if container {
        init_container_logging(tracing_level, log_channel)
    } else {
        match std::process::id() {
            1 => init_pid1_logging(tracing_level, log_channel),
            _ => init_daemon_logging(tracing_level, log_channel),
        }
    }
}

/// Build the layer that sends tracing events to the Observe log stream.
fn broadcast_layer<S>(
    log_channel: LogChannel,
    tracing_level: Level,
) -> impl Layer<S>
where
    S: tracing::Subscriber
        + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .json()
        .with_writer(log_channel)
        .with_filter(EnvFilter::new(format!(
            "auraed={tracing_level},auraed::observe=off"
        )))
}

fn init_container_logging(
    tracing_level: Level,
    log_channel: LogChannel,
) -> Result<(), LoggingError> {
    info!("initializing container logging");

    let stdout_layer = Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(broadcast_layer(log_channel, tracing_level))
        .try_init()
        .map_err(|e| e.into())
}

/// When we run as a daemon, log to stdout and syslog.
fn init_daemon_logging(
    tracing_level: Level,
    log_channel: LogChannel,
) -> Result<(), LoggingError> {
    info!("initializing syslog logging");

    let syslog_identity = c"auraed";
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
    let stdout_layer = Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(syslog_layer)
        .with(stdout_layer)
        .with(broadcast_layer(log_channel, tracing_level))
        .try_init()
        .map_err(|e| e.into())
}

fn init_pid1_logging(
    tracing_level: Level,
    log_channel: LogChannel,
) -> Result<(), LoggingError> {
    info!("initializing pid1 logging");

    let stdout_layer = Layer::with_filter(
        tracing_subscriber::fmt::layer().compact(),
        EnvFilter::new(format!("auraed={tracing_level}")),
    );

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(broadcast_layer(log_channel, tracing_level))
        .try_init()
        .map_err(|e| LoggingError::SetupFailure { source: Box::new(e) })
}
