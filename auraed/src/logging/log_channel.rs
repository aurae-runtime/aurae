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

/// Capacity of the broadcast ring and each gRPC forwarding queue.
///
/// This preserves the original effective ring size: Tokio rounded the former
/// capacity of 40 up to 64. Using that exact power of two makes the allocation
/// explicit and lets each forwarding queue absorb one full ring of messages.
pub(crate) const LOG_STREAM_CAPACITY: usize = 64;

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
    pub fn new(name: impl Into<String>) -> LogChannel {
        let (tx, _) = broadcast::channel(LOG_STREAM_CAPACITY);
        LogChannel { name: name.into(), tx }
    }

    /// Getter for consumer channel
    pub fn subscribe(&self) -> Receiver<LogItem> {
        self.tx.subscribe()
    }

    /// Borrows the producer side for the tracing writer.
    pub(crate) fn sender(&self) -> &Sender<LogItem> {
        &self.tx
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

    #[tokio::test]
    async fn test_ringbuffer_queue() {
        let channel = LogChannel::new("Test");
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
