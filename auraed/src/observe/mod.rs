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

// @todo @krisnova remove this once logging is futher along
#![allow(dead_code)]

tonic::include_proto!("observe");

use crate::meta;
use crate::observe::observe_server::Observe;
use crossbeam::channel::Receiver;
use futures::executor::block_on;
use log::info;
use std::thread;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

/// The server side implementation of the Observe subsystem.
#[derive(Debug, Clone)]
pub struct ObserveService {
    consumer_aurae_logger: Receiver<LogItem>,
    sub_process_consumer_list: Vec<Receiver<LogItem>>,
}

impl ObserveService {
    pub fn new(consumer_aurae_logger: Receiver<LogItem>) -> ObserveService {
        ObserveService {
            consumer_aurae_logger,
            sub_process_consumer_list: Vec::<Receiver<LogItem>>::new(),
        }
    }

    pub fn register_channel(&mut self, consumer: Receiver<LogItem>) {
        info!("Added new channel");
        self.sub_process_consumer_list.push(consumer);
    }
}

#[tonic::async_trait]
impl Observe for ObserveService {
    async fn status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let meta = meta::AuraeMeta {
            name: "UNKNOWN_NAME".to_string(),
            message: "UNKNOWN_MESSAGE".to_string(),
        };
        let response = StatusResponse { meta: Some(meta) };
        Ok(Response::new(response))
    }
    type GetAuraeDaemonLogStreamStream =
        ReceiverStream<Result<LogItem, Status>>;

    async fn get_aurae_daemon_log_stream(
        &self,
        _request: Request<GetAuraeDaemonLogStreamRequest>,
    ) -> Result<Response<Self::GetAuraeDaemonLogStreamStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<LogItem, Status>>(4);

        let log_consumer = self.consumer_aurae_logger.clone();

        thread::spawn(move || {
            for i in log_consumer.into_iter() {
                if block_on(tx.send(Ok(i))).is_err() {
                    // TODO: error handling. Warning: recursively logging if error message is also send to this grpc api endpoint
                    //  .. thus disabled logging here.
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetSubProcessStreamStream = ReceiverStream<Result<LogItem, Status>>;

    async fn get_sub_process_stream(
        &self,
        request: Request<GetSubProcessStreamRequest>,
    ) -> Result<Response<Self::GetSubProcessStreamStream>, Status> {
        let requested_channel = request.get_ref().channel_type;
        let requested_pid = request.get_ref().process_id;

        println!("Requested Channel {}", requested_channel);
        println!("Requested Process ID {}", requested_pid);

        let (_tx, rx) = mpsc::channel::<Result<LogItem, Status>>(4);

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
