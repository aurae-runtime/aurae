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

use super::cgroup_cache;
use super::error::ObserveServiceError;
use super::observed_event_stream::ObservedEventStream;
use super::proc_cache::{ProcCache, ProcfsProcessInfo};
use crate::ebpf::tracepoint::PerfEventBroadcast;
use crate::logging::log_channel::{LOG_STREAM_CAPACITY, LogChannel};
use aurae_ebpf_shared::{ForkedProcess, ProcessExit, Signal};
use cgroup_cache::CgroupCache;
use proto::observe::{
    GetAuraeDaemonLogStreamRequest, GetAuraeDaemonLogStreamResponse,
    GetPosixSignalsStreamRequest, GetPosixSignalsStreamResponse,
    GetSubProcessStreamRequest, GetSubProcessStreamResponse, LogChannelType,
    LogItem, Signal as PosixSignal, WorkloadType, observe_service_server,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, broadcast::Receiver, broadcast::error::RecvError};
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, info, instrument};

#[derive(Debug, Clone)]
pub struct ObserveService {
    aurae_logger: LogChannel,
    cgroup_cache: CgroupCache,
    proc_cache: Option<ProcCache>,
    posix_signals: Option<PerfEventBroadcast<Signal>>,
    sub_process_consumer_list:
        Arc<Mutex<HashMap<i32, HashMap<LogChannelType, LogChannel>>>>,
    log_stream_shutdown: watch::Sender<bool>,
}

type PerfEvents = (
    Option<PerfEventBroadcast<ForkedProcess>>,
    Option<PerfEventBroadcast<ProcessExit>>,
    Option<PerfEventBroadcast<Signal>>,
);

impl ObserveService {
    pub fn new(aurae_logger: LogChannel, perf_events: PerfEvents) -> Self {
        let (log_stream_shutdown, _) = watch::channel(false);
        let proc_cache = match perf_events {
            (Some(f), Some(e), _) => Some(ProcCache::new(
                Duration::from_secs(60),
                Duration::from_secs(60),
                f,
                e,
                ProcfsProcessInfo {},
            )),
            _ => None,
        };
        Self {
            aurae_logger,
            cgroup_cache: CgroupCache::new("/sys/fs/cgroup".into()),
            proc_cache,
            posix_signals: perf_events.2,
            sub_process_consumer_list: Arc::new(Mutex::new(HashMap::new())),
            log_stream_shutdown,
        }
    }

    pub(crate) fn shutdown_log_streams(&self) {
        let _ = self.log_stream_shutdown.send_replace(true);
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

    #[instrument(skip(self))]
    fn get_posix_signals_stream(
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

    /// Bridge a broadcast `LogItem` consumer to a gRPC stream. Lagged
    /// consumers skip dropped items and resume with the oldest item still in
    /// the broadcast ring. The forwarder exits when the client disconnects or
    /// all senders are dropped.
    fn spawn_log_forwarder<R: Send + 'static>(
        &self,
        mut consumer: Receiver<LogItem>,
        wrap: impl Fn(LogItem) -> R + Send + 'static,
    ) -> ReceiverStream<Result<R, Status>> {
        let (tx, rx) = mpsc::channel::<Result<R, Status>>(LOG_STREAM_CAPACITY);
        let mut shutdown = self.log_stream_shutdown.subscribe();

        // The broadcast layer's EnvFilter excludes events with target
        // `auraed::observe`, so any tracing emitted from inside this task
        // does not feed back into the consumer.
        let _ignored = tokio::spawn(async move {
            loop {
                let item = tokio::select! {
                    _ = shutdown.wait_for(|shutdown| *shutdown) => break,
                    _ = tx.closed() => break,
                    item = consumer.recv() => item,
                };
                match item {
                    Ok(item) => {
                        tokio::select! {
                            _ = shutdown.wait_for(|shutdown| *shutdown) => break,
                            result = tx.send(Ok(wrap(item))) => {
                                if result.is_err() {
                                    break; // gRPC client gone
                                }
                            }
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        });

        ReceiverStream::new(rx)
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

#[cfg(test)]
impl ObserveService {
    pub async fn has_sub_process_channel(
        &self,
        pid: i32,
        channel_type: LogChannelType,
    ) -> bool {
        let consumer_list = self.sub_process_consumer_list.lock().await;
        consumer_list
            .get(&pid)
            .and_then(|channels| channels.get(&channel_type))
            .is_some()
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
        let consumer = self.get_aurae_daemon_log_stream();
        Ok(Response::new(self.spawn_log_forwarder(consumer, |item| {
            GetAuraeDaemonLogStreamResponse { item: Some(item) }
        })))
    }

    type GetSubProcessStreamStream =
        ReceiverStream<Result<GetSubProcessStreamResponse, Status>>;

    async fn get_sub_process_stream(
        &self,
        request: Request<GetSubProcessStreamRequest>,
    ) -> Result<Response<Self::GetSubProcessStreamStream>, Status> {
        let GetSubProcessStreamRequest { process_id: pid, channel_type } =
            request.into_inner();
        let channel = LogChannelType::try_from(channel_type).map_err(|_| {
            ObserveServiceError::InvalidLogChannelType { channel_type }
        })?;

        debug!("get_sub_process_stream channel={channel:?} pid={pid}");

        let consumer = {
            let mut consumer_list = self.sub_process_consumer_list.lock().await;
            consumer_list
                .get_mut(&pid)
                .ok_or(ObserveServiceError::NoChannelsForPid { pid })?
                .get_mut(&channel)
                .ok_or(ObserveServiceError::ChannelNotRegistered {
                    pid,
                    channel_type: channel,
                })?
                .subscribe()
        };

        Ok(Response::new(self.spawn_log_forwarder(consumer, |item| {
            GetSubProcessStreamResponse { item: Some(item) }
        })))
    }

    type GetPosixSignalsStreamStream =
        ReceiverStream<Result<GetPosixSignalsStreamResponse, Status>>;

    async fn get_posix_signals_stream(
        &self,
        request: Request<GetPosixSignalsStreamRequest>,
    ) -> Result<Response<Self::GetPosixSignalsStreamStream>, Status> {
        if self.posix_signals.is_none() {
            return Err(Status::unimplemented(
                "GetPosixSignalStream is not implemented for nested Aurae daemons",
            ));
        }

        Ok(Response::new(self.get_posix_signals_stream(
            request.into_inner().workload.map(|w| (w.workload_type(), w.id)),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::log_channel::LogChannel;
    use proto::observe::LogChannelType;

    #[tokio::test]
    async fn test_register_sub_process_channel_success() {
        let svc = ObserveService::new(
            LogChannel::new(String::from("auraed")),
            (None, None, None),
        );
        assert!(
            svc.register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok()
        );

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_register_sub_process_channel_duplicate_error() {
        let svc = ObserveService::new(
            LogChannel::new(String::from("auraed")),
            (None, None, None),
        );
        assert!(
            svc.register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok()
        );
        assert!(
            svc.register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("bar"))
            )
            .await
            .is_err()
        );

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_success() {
        let svc = ObserveService::new(
            LogChannel::new(String::from("auraed")),
            (None, None, None),
        );
        assert!(
            svc.register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok()
        );
        assert!(
            svc.unregister_sub_process_channel(42, LogChannelType::Stdout)
                .await
                .is_ok()
        );

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_no_pid_error() {
        let svc = ObserveService::new(
            LogChannel::new(String::from("auraed")),
            (None, None, None),
        );
        assert!(
            svc.unregister_sub_process_channel(42, LogChannelType::Stdout)
                .await
                .is_err()
        );

        svc.sub_process_consumer_list.lock().await.clear();
    }

    #[tokio::test]
    async fn test_unregister_sub_process_channel_no_channel_type_error() {
        let svc = ObserveService::new(
            LogChannel::new(String::from("auraed")),
            (None, None, None),
        );
        assert!(
            svc.register_sub_process_channel(
                42,
                LogChannelType::Stdout,
                LogChannel::new(String::from("foo"))
            )
            .await
            .is_ok()
        );
        assert!(
            svc.unregister_sub_process_channel(42, LogChannelType::Stderr)
                .await
                .is_err()
        );

        svc.sub_process_consumer_list.lock().await.clear();
    }

    /// A lagged consumer resumes with the oldest item still in the ring.
    #[tokio::test]
    async fn lagged_consumer_continues() {
        use tokio_stream::StreamExt as _;

        let aurae_logger = LogChannel::new("auraed");
        let svc = ObserveService::new(aurae_logger.clone(), (None, None, None));

        let mut stream =
            <ObserveService as observe_service_server::ObserveService>::get_aurae_daemon_log_stream(
                &svc,
                Request::new(GetAuraeDaemonLogStreamRequest {}),
            )
            .await
            .expect("handler returned stream")
            .into_inner();

        const BURST: usize = 1024;
        for i in 0..BURST {
            aurae_logger.send(format!("msg {i:04}"));
        }

        let first = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            stream.next(),
        )
        .await
        .expect("did not receive first item before timeout")
        .expect("stream ended unexpectedly")
        .expect("stream item should be Ok");

        let line = first.item.expect("LogItem present").line;
        assert_eq!(line, format!("msg {:04}", BURST - LOG_STREAM_CAPACITY));
    }

    #[tokio::test]
    async fn shutdown_ends_log_stream() {
        use tokio_stream::StreamExt as _;

        let svc =
            ObserveService::new(LogChannel::new("auraed"), (None, None, None));
        let mut stream =
            <ObserveService as observe_service_server::ObserveService>::get_aurae_daemon_log_stream(
                &svc,
                Request::new(GetAuraeDaemonLogStreamRequest {}),
            )
            .await
            .expect("handler returned stream")
            .into_inner();

        svc.shutdown_log_streams();

        let item = tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("stream did not stop before timeout");
        assert!(item.is_none(), "stream should end without an error item");
    }

    #[tokio::test]
    async fn shutdown_stops_backpressured_log_forwarder() {
        let aurae_logger = LogChannel::new("auraed");
        let svc = ObserveService::new(aurae_logger.clone(), (None, None, None));
        let _stream =
            <ObserveService as observe_service_server::ObserveService>::get_aurae_daemon_log_stream(
                &svc,
                Request::new(GetAuraeDaemonLogStreamRequest {}),
            )
            .await
            .expect("handler returned stream")
            .into_inner();

        for i in 0..=LOG_STREAM_CAPACITY {
            aurae_logger.send(format!("msg {i}"));
            tokio::task::yield_now().await;
        }

        svc.shutdown_log_streams();

        tokio::time::timeout(Duration::from_secs(2), async {
            while aurae_logger.sender().receiver_count() != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("forwarder remained subscribed after shutdown");
    }
}
