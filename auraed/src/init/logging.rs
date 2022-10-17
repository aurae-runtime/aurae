use crate::{logging::streamlogger::StreamLogger, observe::LogItem};
use crossbeam::channel::Sender;
use log::{Level, SetLoggerError};
use simplelog::SimpleLogger;
use syslog::{BasicLogger, Facility, Formatter3164};

const AURAED_SYSLOG_NAME: &str = "auraed";

#[derive(thiserror::Error, Debug)]
pub(crate) enum LoggingError {
    #[error("Unable to connect to syslog: {0}")]
    SysLogConnectionFailure(SetLoggerError),
    #[error("Unable to setup syslog: {0}")]
    SysLogSetupFailure(SetLoggerError),
}

pub(crate) fn init(
    logger_level: Level,
    producer: Sender<LogItem>,
) -> Result<(), LoggingError> {
    match std::process::id() {
        1 => init_pid1_logging(logger_level, producer),
        _ => init_syslog_logging(logger_level, producer),
    }
}

fn init_syslog_logging(
    logger_level: Level,
    producer: Sender<LogItem>,
) -> Result<(), LoggingError> {
    // Syslog formatter
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: AURAED_SYSLOG_NAME.into(),
        pid: 0,
    };

    // Initialize the logger
    let logger_simple = create_logger_simple(logger_level);
    let logger_stream = Box::new(StreamLogger::new(producer));

    let logger_syslog = match syslog::unix(formatter) {
        Ok(log_val) => log_val,
        Err(e) => {
            panic!("Unable to setup syslog: {:?}", e);
        }
    };

    multi_log::MultiLogger::init(
        vec![
            logger_simple,
            Box::new(BasicLogger::new(logger_syslog)),
            logger_stream,
        ],
        logger_level,
    )
    .map_err(LoggingError::SysLogSetupFailure)
}

// To discuss here https://github.com/aurae-runtime/auraed/issues/24:
//      The "syslog" logger requires unix sockets.
//      Syslog assumes that either /dev/log or /var/run/syslog are available [1].
//      We need to discuss if there is a use case to log via unix sockets,
//      other than fullfill the requirement of syslog crate.
//      For now, auraed distinguishes between pid1 system and local (dev environment) logging.
//      [1] https://docs.rs/syslog/latest/src/syslog/lib.rs.html#232-243
fn init_pid1_logging(
    logger_level: Level,
    producer: Sender<LogItem>,
) -> Result<(), LoggingError> {
    // Initialize the logger
    let logger_simple = create_logger_simple(logger_level);

    let logger_stream = Box::new(StreamLogger::new(producer));

    multi_log::MultiLogger::init(
        vec![logger_simple, logger_stream],
        logger_level,
    )
    .map_err(LoggingError::SysLogConnectionFailure)
}

fn create_logger_simple(logger_level: Level) -> Box<SimpleLogger> {
    SimpleLogger::new(
        logger_level.to_level_filter(),
        simplelog::Config::default(),
    )
}
