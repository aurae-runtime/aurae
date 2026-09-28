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

//! `tracing_subscriber::fmt::MakeWriter` impl for `LogChannel`.
//!
//! Wiring `LogChannel` directly as the writer type means a layer is built
//! by passing the channel through `with_writer`:
//!
//! ```ignore
//! tracing_subscriber::fmt::layer().with_writer(log_channel.clone())
//! ```
//!
//! `MakeWriter::make_writer()` is invoked once per `tracing` event, the
//! returned `BroadcastWriter` accumulates the formatted bytes in a per-event
//! buffer, and `Drop` flushes one `LogItem` into the broadcast.

use super::{get_timestamp_sec, log_channel::LogChannel};
use proto::observe::LogItem;
use std::io;
use tokio::sync::broadcast::Sender;
use tracing_subscriber::fmt::MakeWriter;

impl<'a> MakeWriter<'a> for LogChannel {
    type Writer = BroadcastWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        BroadcastWriter {
            tx: self.sender(),
            channel: &self.name,
            buf: Vec::new(),
        }
    }
}

/// Per-event writer. Accumulates bytes and flushes a single `LogItem` on drop.
#[derive(Debug)]
pub struct BroadcastWriter<'a> {
    tx: &'a Sender<LogItem>,
    channel: &'a str,
    buf: Vec<u8>,
}

impl<'a> io::Write for BroadcastWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Drop for BroadcastWriter<'a> {
    fn drop(&mut self) {
        if self.buf.is_empty() || self.tx.receiver_count() == 0 {
            return;
        }
        // Reuse the buffer's allocation: fmt::Layer emits one trailing '\n'
        // per event, which we drop in place, and `String::from_utf8` takes
        // ownership of the `Vec` without copying on the (overwhelmingly
        // common) valid-UTF-8 path.
        let mut buf = std::mem::take(&mut self.buf);
        while buf.last() == Some(&b'\n') {
            let _ = buf.pop();
        }
        let line = String::from_utf8(buf).unwrap_or_else(|e| {
            String::from_utf8_lossy(e.as_bytes()).into_owned()
        });
        let _ = self.tx.send(LogItem {
            channel: self.channel.to_owned(),
            line,
            timestamp: get_timestamp_sec(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tokio::sync::broadcast::error::TryRecvError;
    use tracing::subscriber::with_default;
    use tracing_subscriber::{
        EnvFilter, Layer, layer::SubscriberExt, registry,
    };

    fn build_subscriber(
        channel: &LogChannel,
        filter: &str,
    ) -> impl tracing::Subscriber + Send + Sync {
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(channel.clone())
            .with_filter(EnvFilter::new(filter));
        registry().with(layer)
    }

    fn drain(
        rx: &mut tokio::sync::broadcast::Receiver<LogItem>,
    ) -> Vec<LogItem> {
        let mut out = Vec::new();
        loop {
            match rx.try_recv() {
                Ok(item) => out.push(item),
                Err(TryRecvError::Empty | TryRecvError::Closed) => break,
                Err(TryRecvError::Lagged(_)) => {
                    panic!("unexpected Lagged in test")
                }
            }
        }
        out
    }

    fn parse_line(line: &str) -> Value {
        serde_json::from_str(line).expect("valid JSON log line")
    }

    #[test]
    fn emits_exactly_one_log_item_per_event_as_json() {
        let channel = LogChannel::new("test-channel");
        let mut rx = channel.subscribe();
        let subscriber = build_subscriber(&channel, "info");

        let message = "streamed log message";
        let before = get_timestamp_sec();
        with_default(subscriber, || {
            tracing::info!(target: "auraed::test", "{message}");
        });
        let after = get_timestamp_sec();

        let items = drain(&mut rx);
        assert_eq!(items.len(), 1, "expected exactly one LogItem");
        let item = &items[0];

        assert_eq!(item.channel, "test-channel");
        assert!(
            item.timestamp >= before && item.timestamp <= after,
            "timestamp {} not in [{before}, {after}]",
            item.timestamp
        );
        let json = parse_line(&item.line);
        assert_eq!(json["level"].as_str(), Some("INFO"), "json: {json}");
        assert_eq!(
            json["target"].as_str(),
            Some("auraed::test"),
            "json: {json}"
        );
        assert_eq!(
            json["fields"]["message"].as_str(),
            Some(message),
            "json: {json}"
        );
    }

    #[test]
    fn env_filter_excludes_observe_target() {
        let channel = LogChannel::new("test");
        let mut rx = channel.subscribe();
        let subscriber = build_subscriber(&channel, "info,auraed::observe=off");

        with_default(subscriber, || {
            tracing::info!(target: "auraed::observe", "suppressed");
            tracing::info!(target: "auraed::cells", "visible");
        });

        let items = drain(&mut rx);
        assert_eq!(items.len(), 1, "exactly one event should pass the filter");

        let json = parse_line(&items[0].line);
        assert_eq!(json["target"].as_str(), Some("auraed::cells"));
        assert_eq!(json["fields"]["message"].as_str(), Some("visible"));
    }

    #[test]
    fn short_circuits_when_no_subscribers() {
        let channel = LogChannel::new("test");
        let subscriber = build_subscriber(&channel, "info");
        assert_eq!(channel.sender().receiver_count(), 0);

        with_default(subscriber, || {
            for _ in 0..50 {
                tracing::info!("no subscribers");
            }
        });

        let mut rx = channel.subscribe();
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
    }
}
