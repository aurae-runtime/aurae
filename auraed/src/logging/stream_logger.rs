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

use log::Log;
use proto::observe::LogItem;
use tokio::sync::broadcast::Sender;

/// Sends log messages generated in rust code to the logging channel
/// The logging channel is consumed by the observe API
#[derive(Debug)]
pub struct StreamLogger {
    /// Channel used to send log messages to grpc API
    pub producer: Sender<LogItem>,
}

impl StreamLogger {
    #[allow(unused)]
    /// Constructor requires channel between api and logger
    pub fn new(producer: Sender<LogItem>) -> StreamLogger {
        StreamLogger { producer }
    }
}

impl Log for StreamLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // send returns an Err if there are no receivers. We ignore that.
        let _ = self.producer.send(LogItem {
            channel: "rust-logs".to_string(),
            line: format!(
                "{}:{} -- {}",
                record.level(),
                record.target(),
                record.args()
            ),
            timestamp: 0,
        });
    }

    fn flush(&self) {}
}