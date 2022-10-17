/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use std::time::SystemTime;

use crate::observe::LogItem;
use crossbeam::channel::{bounded, Receiver, Sender};
use log::{error, trace};

use super::get_timestamp_sec;

/// Abstraction Layer for one log generating entity
/// LogChannel provides channels between Log producers and log consumers
#[derive(Debug)]
pub struct LogChannel {
    producer: Sender<LogItem>,
    consumer: Receiver<LogItem>,
    name: String,
}

impl LogChannel {
    /// Constructor creating the channel for log communication
    pub fn new(name: &str) -> LogChannel {
        // TODO: decide for a cap. 40 is arbitrary 
        let (producer, consumer) = bounded(40);
        LogChannel { producer, consumer, name: name.to_string() }
    }
    /// Getter for producer channel
    pub fn get_producer(&self) -> Sender<LogItem> {
        self.producer.clone()
    }

    /// Getter for consumer channel
    pub fn get_consumer(&self) -> Receiver<LogItem> {
        self.consumer.clone()
    }

    /// Wrapper that sends a log line to the channel
    pub fn log_line(producer: Sender<LogItem>, line: &str) {
        match producer.send(LogItem {
            channel: "unknown".to_string(),
            line: line.to_string(),
            // TODO: milliseconds type in protobuf requires 128bit type
            timestamp: get_timestamp_sec(),
        }) {
            Ok(_) => {}
            Err(e) => {
                error!("Error! {:?}", e);
            }
        }
    }

    // Receives a message from the channel
    // multiple consumer possible 
    fn consume_line(consumer: Receiver<LogItem>) -> Option<LogItem> {
        match consumer.recv() {
            Ok(val) => {
                return Some(val);
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use log::Level;
    use simplelog::SimpleLogger;

    use super::*;

    fn init_logging() {
        let logger_simple = SimpleLogger::new(
            Level::Trace.to_level_filter(),
            simplelog::Config::default(),
        );

        multi_log::MultiLogger::init(vec![logger_simple], Level::Trace)
            .unwrap();
    }

    #[tokio::test]
    async fn test_ringbuffer_queue() {
        init_logging();
        let lrb = LogChannel::new("Test");
        let prod = lrb.get_producer();

        LogChannel::log_line(prod.clone(), "hello");
        LogChannel::log_line(prod.clone(), "aurae");
        LogChannel::log_line(prod.clone(), "bye");

        let consumer = lrb.get_consumer();

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "hello");

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "aurae");

        let cur_item = LogChannel::consume_line(consumer.clone());
        assert!(cur_item.is_some());
        assert_eq!(cur_item.unwrap().line, "bye");
    }
}
