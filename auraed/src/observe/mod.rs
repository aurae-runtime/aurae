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

// @todo @krisnova remove this once logging is further along
#![allow(dead_code)]

use aurae_proto::observe::{
    observe_service_server::ObserveService, GetAuraeDaemonLogStreamRequest,
    GetSubProcessStreamRequest, LogItem,
};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::logging::log_channel::LogChannel;

/// The server side implementation of the ObserveService subsystem.
#[derive(Debug)]
pub(crate) struct ObserveServiceServer {
    aurae_logger: Arc<LogChannel>,
    sub_process_consumer_list: Vec<Receiver<LogItem>>,
}

impl ObserveServiceServer {
    pub fn new(aurae_logger: Arc<LogChannel>) -> ObserveServiceServer {
        ObserveServiceServer {
            aurae_logger,
            sub_process_consumer_list: Vec::<Receiver<LogItem>>::new(),
        }
    }

    pub fn register_channel(&mut self, consumer: Receiver<LogItem>) {
        info!("Added new channel");
        self.sub_process_consumer_list.push(consumer);
    }
}

#[tonic::async_trait]
impl ObserveService for ObserveServiceServer {
    type GetAuraeDaemonLogStreamStream =
        ReceiverStream<Result<LogItem, Status>>;

    async fn get_aurae_daemon_log_stream(
        &self,
        _request: Request<GetAuraeDaemonLogStreamRequest>,
    ) -> Result<Response<Self::GetAuraeDaemonLogStreamStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<LogItem, Status>>(4);

        let mut log_consumer = self.aurae_logger.subscribe();

        // TODO: error handling. Warning: recursively logging if error message is also send to this grpc api endpoint
        //  .. thus disabled logging here.
        let _ = tokio::spawn(async move {
            // Log consumer will error if:
            //  the producer is closed (no more logs)
            //  the receiver is lagging
            while let Ok(log_item) = log_consumer.recv().await {
                if tx.send(Ok(log_item)).await.is_err() {
                    // receiver is gone
                    break;
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
