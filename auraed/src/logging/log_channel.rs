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

use super::get_timestamp_sec;
use proto::observe::LogItem;
use tokio::sync::broadcast::{self, Receiver, Sender};

/// Abstraction Layer for one log generating entity
/// LogChannel provides channels between Log producers and log consumers
#[derive(Clone, Debug)]
pub struct LogChannel {
    /// The human readable (public) name for this log channel.
    pub name: String,
    tx: Sender<LogItem>,
}

impl LogChannel {
    /// Constructor creating the channel for log communication
    pub fn new(name: String) -> LogChannel {
        // TODO: decide for a cap. 40 is arbitrary
        let (tx, _) = broadcast::channel(40);
        LogChannel { name, tx }
    }

    /// Getter for consumer channel
    pub fn subscribe(&self) -> Receiver<LogItem> {
        self.tx.subscribe()
    }

    /// Wrapper that sends a log line to the channel
    pub fn send(&self, line: String) {
        // send returns an Err if there are no receivers. We ignore that.
        let _ = self.tx.send(LogItem {
            channel: self.name.clone(),
            line,
            // TODO: milliseconds type in protobuf requires 128bit type
            timestamp: get_timestamp_sec(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Level;
    use simplelog::SimpleLogger;

    fn init_logging() {
        let logger_simple = SimpleLogger::new(
            Level::Trace.to_level_filter(),
            simplelog::Config::default(),
        );

        multi_log::MultiLogger::init(vec![logger_simple], Level::Trace)
            .expect("failed to initialize logger");
    }

    #[tokio::test]
    async fn test_ringbuffer_queue() {
        init_logging();
        let channel = LogChannel::new("Test".into());
        let mut rx = channel.subscribe();

        channel.send("hello".into());
        channel.send("aurae".into());
        channel.send("bye".into());

        let cur_item = rx.recv().await.ok();
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "hello".to_string());

        let cur_item = rx.recv().await.ok();
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "aurae".to_string());

        let cur_item = rx.recv().await.ok();
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "bye".to_string());
    }
}