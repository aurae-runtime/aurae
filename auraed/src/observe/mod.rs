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

// @todo @krisnova remove this once logging is further along
#![allow(dead_code)]

mod cgroup_cache;

use aurae_ebpf_shared::Signal;
use proto::observe::{
    observe_service_server, GetAuraeDaemonLogStreamRequest,
    GetAuraeDaemonLogStreamResponse, GetPosixSignalsStreamRequest,
    GetPosixSignalsStreamResponse, GetSubProcessStreamRequest,
    GetSubProcessStreamResponse, LogItem, Signal as PosixSignal, WorkloadType,
};
use std::{ffi::OsString, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::{broadcast::Receiver, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::info;

use crate::{
    ebpf::perf_event_listener::PerfEventListener,
    logging::log_channel::LogChannel,
};

use self::cgroup_cache::CgroupCache;

//TODO (jeroensoeters) move this service to its own module
pub struct ObserveService {
    aurae_logger: Arc<LogChannel>,
    cgroup_cache: Arc<Mutex<CgroupCache>>,
    posix_signals: Option<PerfEventListener<Signal>>,
    sub_process_consumer_list: Vec<Receiver<LogItem>>,
}

impl ObserveService {
    pub fn new(
        aurae_logger: Arc<LogChannel>,
        posix_signals: Option<PerfEventListener<Signal>>,
    ) -> Self {
        Self {
            aurae_logger,
            cgroup_cache: Arc::new(Mutex::new(cgroup_cache::CgroupCache::new(
                OsString::from("/sys/fs/cgroup"),
            ))),
            posix_signals,
            sub_process_consumer_list: Vec::<Receiver<LogItem>>::new(),
        }
    }

    pub fn register_channel(&mut self, consumer: Receiver<LogItem>) {
        info!("Added new channel");
        self.sub_process_consumer_list.push(consumer);
    }

    fn get_aurae_daemon_log_stream(&self) -> Receiver<LogItem> {
        self.aurae_logger.subscribe()
    }

    async fn get_posix_signals_stream(
        &self,
        filter: Option<(WorkloadType, String)>,
    ) -> ReceiverStream<Result<GetPosixSignalsStreamResponse, Status>> {
        let (tx, rx) =
            mpsc::channel::<Result<GetPosixSignalsStreamResponse, Status>>(4);

        let mut posix_signals = self
            .posix_signals
            .as_ref()
            .expect("posix signal perf event listener")
            .subscribe();

        let thread_cache = self.cgroup_cache.clone();
        let _ignored = tokio::spawn(async move {
            while let Ok(signal) = posix_signals.recv().await {
                let accept = match filter {
                    Some((WorkloadType::Cell, ref id)) => {
                        let cgroup_path = format!("/sys/fs/cgroup/{}/_", id);
                        let mut cache = thread_cache.lock().await;
                        cache
                            .get(signal.cgroupid)
                            .map(|path| path.eq_ignore_ascii_case(cgroup_path))
                            .unwrap_or(false)
                    }
                    _ => true,
                };
                if accept {
                    let resp = GetPosixSignalsStreamResponse {
                        signal: Some(PosixSignal {
                            signal: signal.signr,
                            process_id: i64::from(signal.pid),
                        }),
                    };
                    if tx.send(Ok(resp)).await.is_err() {
                        // receiver is gone
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
}

#[tonic::async_trait]
impl observe_service_server::ObserveService for ObserveService {
    type GetAuraeDaemonLogStreamStream =
        ReceiverStream<Result<GetAuraeDaemonLogStreamResponse, Status>>;

    async fn get_aurae_daemon_log_stream(
        &self,
        _request: Request<GetAuraeDaemonLogStreamRequest>,
    ) -> Result<Response<Self::GetAuraeDaemonLogStreamStream>, Status> {
        let (tx, rx) =
            mpsc::channel::<Result<GetAuraeDaemonLogStreamResponse, Status>>(4);
        let mut log_consumer = self.get_aurae_daemon_log_stream();

        // TODO: error handling. Warning: recursively logging if error message is also send to this grpc api endpoint
        //  .. thus disabled logging here.
        let _ignored = tokio::spawn(async move {
            // Log consumer will error if:
            //  the producer is closed (no more logs)
            //  the receiver is lagging
            while let Ok(log_item) = log_consumer.recv().await {
                let resp =
                    GetAuraeDaemonLogStreamResponse { item: Some(log_item) };
                if tx.send(Ok(resp)).await.is_err() {
                    // receiver is gone
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetSubProcessStreamStream =
        ReceiverStream<Result<GetSubProcessStreamResponse, Status>>;

    async fn get_sub_process_stream(
        &self,
        request: Request<GetSubProcessStreamRequest>,
    ) -> Result<Response<Self::GetSubProcessStreamStream>, Status> {
        let requested_channel = request.get_ref().channel_type;
        let requested_pid = request.get_ref().process_id;

        println!("Requested Channel {requested_channel}");
        println!("Requested Process ID {requested_pid}");

        let (_tx, rx) =
            mpsc::channel::<Result<GetSubProcessStreamResponse, Status>>(4);

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetPosixSignalsStreamStream =
        ReceiverStream<Result<GetPosixSignalsStreamResponse, Status>>;

    async fn get_posix_signals_stream(
        &self,
        request: Request<GetPosixSignalsStreamRequest>,
    ) -> Result<Response<Self::GetPosixSignalsStreamStream>, Status> {
        if self.posix_signals.is_none() {
            return Err(Status::unimplemented("GetPosixSignalStream is not implemented for nested Aurae daemons"));
        }

        Ok(Response::new(
            self.get_posix_signals_stream(
                request
                    .into_inner()
                    .workload
                    .map(|w| (w.workload_type(), w.id)),
            )
            .await,
        ))
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::ebpf::loader::BpfLoader;
//     use std::{process::Command, time::Duration};
//     use test_helpers::*;
//     use tokio::sync::Mutex;

//     #[tokio::test]
//     #[ignore] // Cannot run this in CI as we don't want to install eBPF probes on the GHA hosts :)
//     async fn test_intercept_posix_signals() {
//         skip_if_not_root!("test_intercept_posix_signals");
//         let bpf_loader = &mut BpfLoader::new();
//         let signals_listener = bpf_loader
//             .read_and_load_tracepoint_signal_signal_generate()
//             .expect("failed to attach signals tracepoint");

//         let service = ObserveService::new(
//             Arc::new(LogChannel::new(String::from("unused"))),
//             Some(signals_listener),
//         );

//         let mut signals = service.get_posix_signals_stream();

//         let intercepted = Arc::new(Mutex::new(Vec::new()));
//         let intercepted_in_thread = intercepted.clone();
//         let _ = tokio::spawn(async move {
//             while let Ok(s) = signals.recv().await {
//                 let mut guard = intercepted_in_thread.lock().await;
//                 guard.push(s);
//             }
//         });

//         let mut child = Command::new("sleep")
//             .arg("400")
//             .spawn()
//             .expect("failed to execute child");
//         let pid = child.id();
//         child.kill().expect("failed to kill child");

//         let expected_signal = Signal { pid: pid, signr: 9 };

//         tokio::time::sleep(Duration::from_millis(500)).await;

//         let intercepted_local = intercepted.clone();
//         let guard = intercepted_local.lock().await;

//         assert!(guard.contains(&expected_signal), "signal not found");
//     }
// }
