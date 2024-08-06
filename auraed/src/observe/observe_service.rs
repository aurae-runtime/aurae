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

// @todo @krisnova remove this once logging is further along
#![allow(dead_code)]
//TODO(jeroen) this warning comes from the `perf_events` arument in ObserveService::new()
#![allow(clippy::type_complexity)]

use super::cgroup_cache;
use super::error::ObserveServiceError;
use super::observed_event_stream::ObservedEventStream;
use super::proc_cache::{ProcCache, ProcfsProcessInfo};
use crate::ebpf::tracepoint::PerfEventBroadcast;
use crate::logging::log_channel::LogChannel;
use aurae_ebpf_shared::{ForkedProcess, ProcessExit, Signal};
use cgroup_cache::CgroupCache;
use proto::observe::{
    observe_service_server, GetAuraeDaemonLogStreamRequest,
    GetAuraeDaemonLogStreamResponse, GetPosixSignalsStreamRequest,
    GetPosixSignalsStreamResponse, GetSubProcessStreamRequest,
    GetSubProcessStreamResponse, LogChannelType, LogItem,
    Signal as PosixSignal, WorkloadType,
};
use std::collections::HashMap;
use std::time::Duration;
use std::{ffi::OsString, sync::Arc};
use tokio::sync::mpsc;
use tokio::sync::{broadcast::Receiver, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ObserveService {
    aurae_logger: Arc<LogChannel>,
    cgroup_cache: Arc<Mutex<CgroupCache>>,
    proc_cache: Option<Arc<Mutex<ProcCache>>>,
    posix_signals: Option<PerfEventBroadcast<Signal>>,
    sub_process_consumer_list:
        Arc<Mutex<HashMap<i32, HashMap<LogChannelType, LogChannel>>>>,
}

impl ObserveService {
    pub fn new(
        aurae_logger: Arc<LogChannel>,
        perf_events: (
            Option<PerfEventBroadcast<ForkedProcess>>,
            Option<PerfEventBroadcast<ProcessExit>>,
            Option<PerfEventBroadcast<Signal>>,
        ),
    ) -> Self {
        let proc_cache = match perf_events {
            (Some(f), Some(e), _) => {
                Some(Arc::new(Mutex::new(ProcCache::new(
                    Duration::from_secs(60),
                    Duration::from_secs(60),
                    f,
                    e,
                    ProcfsProcessInfo {},
                ))))
            }
            _ => None,
        };
        Self {
            aurae_logger,
            cgroup_cache: Arc::new(Mutex::new(CgroupCache::new(
                OsString::from("/sys/fs/cgroup"),
            ))),
            proc_cache,
            posix_signals: perf_events.2,
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
        //TODO map err -> gRPC error status
        let events = ObservedEventStream::new(
            self.posix_signals.as_ref().expect("signals"),
        )
        .filter_by_workload(filter)
        .map_pids(self.proc_cache.as_ref().expect("proc_cache").clone())
        .subscribe(map_get_posix_signals_stream_response);

        ReceiverStream::new(events)
    }
}

fn map_get_posix_signals_stream_response(
    signal: Signal,
    pid: i32,
) -> GetPosixSignalsStreamResponse {
    GetPosixSignalsStreamResponse {
        signal: Some(PosixSignal { signal: signal.signum, process_id: pid }),
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
    use super::ObserveService;
    use crate::logging::log_channel::LogChannel;
    use proto::observe::LogChannelType;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_register_sub_process_channel_success() {
        let svc = ObserveService::new(
            Arc::new(LogChannel::new(String::from("auraed"))),
            (None, None, None),
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
            (None, None, None),
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
            (None, None, None),
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
            (None, None, None),
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
            (None, None, None),
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