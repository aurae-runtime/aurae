/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
