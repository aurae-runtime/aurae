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

use super::cgroup_cache;
use crate::{
    ebpf::tracepoint_programs::PerfEventBroadcast,
    logging::log_channel::LogChannel,
};
use aurae_ebpf_shared::Signal;
use cgroup_cache::CgroupCache;
use proto::observe::{
    observe_service_server, GetAuraeDaemonLogStreamRequest,
    GetAuraeDaemonLogStreamResponse, GetPosixSignalsStreamRequest,
    GetPosixSignalsStreamResponse, GetSubProcessStreamRequest,
    GetSubProcessStreamResponse, LogChannelType, LogItem,
    Signal as PosixSignal, WorkloadType,
};
use std::collections::HashMap;
use std::{ffi::OsString, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::{broadcast::Receiver, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::info;
use super::cgroup_cache;
use super::error::ObserveServiceError;
use procfs::process::Process;

#[derive(Debug, Clone)]
pub struct ObserveService {
    aurae_logger: Arc<LogChannel>,
    cgroup_cache: Arc<Mutex<CgroupCache>>,
    posix_signals: Option<PerfEventBroadcast<Signal>>,
    sub_process_consumer_list:
        Arc<Mutex<HashMap<i32, HashMap<LogChannelType, LogChannel>>>>,
}

impl ObserveService {
    pub fn new(
        aurae_logger: Arc<LogChannel>,
        posix_signals: Option<PerfEventBroadcast<Signal>>,
    ) -> Self {
        Self {
            aurae_logger,
            cgroup_cache: Arc::new(Mutex::new(CgroupCache::new(
                OsString::from("/sys/fs/cgroup"),
            ))),
            posix_signals,
            sub_process_consumer_list: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_sub_process_channel(
        &self,
        pid: i32,
        channel_type: LogChannelType,
        channel: LogChannel,
    ) -> Result<(), ObserveServiceError> {
        info!("Registering channel for pid {pid} {channel_type:?}");
        let mut consumer_list = self.sub_process_consumer_list.lock().await;
        if consumer_list.get(&pid).is_none() {
            let _ = consumer_list.insert(pid, HashMap::new());
        }
        if consumer_list
            .get(&pid)
            .expect("pid channels")
            .get(&channel_type)
            .is_some()
        {
            return Err(ObserveServiceError::ChannelAlreadyRegistered {
                pid,
                channel_type,
            });
        }
        let _ = consumer_list
            .get_mut(&pid)
            .expect("pid channels")
            .insert(channel_type, channel);
        Ok(())
    }

    pub async fn unregister_sub_process_channel(
        &self,
        pid: i32,
        channel_type: LogChannelType,
    ) -> Result<(), ObserveServiceError> {
        info!("Unregistering for pid {pid} {channel_type:?}");
        let mut consumer_list = self.sub_process_consumer_list.lock().await;
        if let Some(channels) = consumer_list.get_mut(&pid) {
            if channels.remove(&channel_type).is_none() {
                return Err(ObserveServiceError::ChannelNotRegistered {
                    pid,
                    channel_type,
                });
            }
        } else {
            return Err(ObserveServiceError::NoChannelsForPid { pid });
        }
        Ok(())
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

        // TODO: this panics if eBPF program hasn't been loaded
        let mut posix_signals = self
            .posix_signals
            .as_ref()
            .expect("posix signal perf event listener")
            .subscribe();

        let (match_cgroup_path, cgroup_path) = match filter {
            Some((WorkloadType::Cell, id)) => {
                (true, format!("/sys/fs/cgroup/{id}/_"))
            }
            _ => (false, String::new()),
        };

        let thread_cache = self.cgroup_cache.clone();
        let _ignored = tokio::spawn(async move {
            while let Ok(signal) = posix_signals.recv().await {
                if signal.signum == 9 {
                    let p = Process::new(signal.pid as i32);
                    match p {
                        Ok(pr) => {
                            info!(
                                "pid: {}, nspid {:#?}",
                                pr.pid,
                                pr.status().expect("status").nspid
                            );
                        }
                        Err(_) => {
                            info!("PROCESS NOT FOUND {}", signal.pid);
                        }
                    }
                }

                let accept = !match_cgroup_path || {
                    let mut cache = thread_cache.lock().await;
                    cache
                        .get(signal.cgroup_id)
                        .map(|path| path.eq_ignore_ascii_case(&cgroup_path))
                        .unwrap_or(false)
                };
                if accept {
                    let resp = GetPosixSignalsStreamResponse {
                        signal: Some(PosixSignal {
                            signal: signal.signum,
                            process_id: signal.pid,
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
        let channel = LogChannelType::from_i32(request.get_ref().channel_type)
            .ok_or(ObserveServiceError::InvalidLogChannelType {
                channel_type: request.get_ref().channel_type,
            })?;
        let pid: i32 = request.get_ref().process_id;

        println!("Requested Channel {channel:?}");
        println!("Requested Process ID {pid}");

        let mut log_consumer = {
            let mut consumer_list = self.sub_process_consumer_list.lock().await;
            consumer_list
                .get_mut(&pid)
                .ok_or(ObserveServiceError::NoChannelsForPid { pid })?
                .get_mut(&channel)
                .ok_or(ObserveServiceError::ChannelNotRegistered {
                    pid,
                    channel_type: channel,
                })?
                .clone()
        }
        .subscribe();

        let (tx, rx) =
            mpsc::channel::<Result<GetSubProcessStreamResponse, Status>>(4);

        // TODO: error handling. Warning: recursively logging if error message is also send to this grpc api endpoint
        //  .. thus disabled logging here.
        let _ignored = tokio::spawn(async move {
            // Log consumer will error if:
            //  the producer is closed (no more logs)
            //  the receiver is lagging
            while let Ok(log_item) = log_consumer.recv().await {
                let resp = GetSubProcessStreamResponse { item: Some(log_item) };
                if tx.send(Ok(resp)).await.is_err() {
                    // receiver is gone
                    break;
                }
            }
        });

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use proto::observe::LogChannelType;

    use crate::logging::log_channel::LogChannel;

    use super::ObserveService;

    #[tokio::test]
    async fn test_register_sub_process_channel_success() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            None,
        );
        assert!(svc
            .register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok());

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_register_sub_process_channel_duplicate_error() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            None,
        );
        assert!(svc
            .register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok());
        assert!(svc
            .register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("bar"))
            )
            .await
            .is_err());

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_success() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            None,
        );
        assert!(svc
            .register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok());
        assert!(svc
            .unregister_sub_process_channel(42, LogChannelType::Stdout)
            .await
            .is_ok());

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_no_pid_error() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            None,
        );
        assert!(svc
            .unregister_sub_process_channel(42, LogChannelType::Stdout)
            .await
            .is_err());

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_no_channel_type_error() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            None,
        );
        assert!(svc
            .register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok());
        assert!(svc
            .unregister_sub_process_channel(42, LogChannelType::Stderr)
            .await
            .is_err());

        svc.sub_process_consumer_list.lock().await.clear();
    }
}
