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
use super::{cgroup_cache::CgroupCache, proc_cache::ProcCache};
use crate::ebpf::tracepoint::PerfEventBroadcast;
use aurae_ebpf_shared::{HasCgroup, HasHostPid};
use proto::observe::WorkloadType;
use std::{ffi::OsString, sync::Arc};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use tonic::Status;

const CGROUPFS_ROOT: &str = "/sys/fs/cgroup";

/// Wrapper around `PerfEventBroadvast<T>` that allows for filtering by
/// Aurae workloads and optionally maps host PIDs to namespace PIDs.
pub struct ObservedEventStream<'a, T> {
    source: &'a PerfEventBroadcast<T>,
    workload_filter: Option<(WorkloadType, String)>,
    proc_cache: Option<Arc<Mutex<ProcCache>>>,
    cgroup_cache: Arc<Mutex<CgroupCache>>,
}

impl<'a, T: HasCgroup + HasHostPid + Clone + Send + Sync + 'static>
    ObservedEventStream<'a, T>
{
    pub fn new(source: &'a PerfEventBroadcast<T>) -> Self {
        Self {
            source,
            workload_filter: None,
            proc_cache: None,
            cgroup_cache: Arc::new(Mutex::new(CgroupCache::new(
                OsString::from(CGROUPFS_ROOT),
            ))),
        }
    }

    pub fn filter_by_workload(
        &mut self,
        workload: Option<(WorkloadType, String)>,
    ) -> &mut Self {
        self.workload_filter = workload;
        self
    }

    pub fn map_pids(&mut self, proc_cache: Arc<Mutex<ProcCache>>) -> &mut Self {
        self.proc_cache = Some(proc_cache);
        self
    }

    pub fn subscribe<E: Send + Sync + 'static>(
        &self,
        map_response: fn(T, i32) -> E,
    ) -> Receiver<Result<E, Status>> {
        let (tx, rx) = mpsc::channel(4);

        let (match_cgroup_path, cgroup_path) = match &self.workload_filter {
            Some((WorkloadType::Cell, id)) => {
                (true, format!("/sys/fs/cgroup/{id}/_"))
            }
            _ => (false, String::new()),
        };
        let mut events = self.source.subscribe();

        let cgroup_thread_cache = self.cgroup_cache.clone();
        let proc_thread_cache = self.proc_cache.as_ref().cloned();
        let _ignored = tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                let accept = !match_cgroup_path || {
                    let mut cache = cgroup_thread_cache.lock().await;
                    cache
                        .get(event.cgroup_id())
                        .map(|path| path.eq_ignore_ascii_case(&cgroup_path))
                        .unwrap_or(false)
                };
                if accept {
                    let pid = if let Some(ref proc_cache) = proc_thread_cache {
                        let guard = proc_cache.lock().await;
                        guard
                            .get(event.host_pid())
                            .await
                            .unwrap_or_else(|| event.host_pid())
                    } else {
                        event.host_pid()
                    };

                    if tx.send(Ok(map_response(event, pid))).await.is_err() {
                        // receiver is gone
                        break;
                    }
                }
            }
        });

        rx
    }
}