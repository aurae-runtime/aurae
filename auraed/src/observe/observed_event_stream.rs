use std::{ffi::OsString, sync::Arc};

use aurae_ebpf_shared::CgroupId;
use proto::observe::WorkloadType;
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
use crate::ebpf::tracepoint::PerfEventBroadcast;
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use tonic::Status;

use super::cgroup_cache::CgroupCache;

const CGROUPFS_ROOT: &str = "/sys/fs/cgroup";

pub struct ObservedEventStream<'a, T> {
    source: &'a PerfEventBroadcast<T>,
    workload_filter: Option<(WorkloadType, String)>,
    //map_pids: bool,
    cgroup_cache: Arc<Mutex<CgroupCache>>,
}

impl<'a, T: CgroupId + Clone + Send + 'static> ObservedEventStream<'a, T> {
    pub fn new(source: &'a PerfEventBroadcast<T>) -> Self {
        Self {
            source,
            workload_filter: None,
            //map_pids: false,
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

    pub fn subscribe<E: Send + 'static>(
        &self,
        map_response: fn(T) -> E,
    ) -> Receiver<Result<E, Status>> {
        let (tx, rx) = mpsc::channel(4);

        let (match_cgroup_path, cgroup_path) = match &self.workload_filter {
            Some((WorkloadType::Cell, id)) => {
                (true, format!("/sys/fs/cgroup/{id}/_"))
            }
            _ => (false, String::new()),
        };
        let mut events = self.source.subscribe();

        let thread_cache = self.cgroup_cache.clone();
        let _ignored = tokio::spawn(async move {
            while let Ok(event) = events.recv().await {
                let accept = !match_cgroup_path || {
                    let mut cache = thread_cache.lock().await;
                    cache
                        .get(event.cgroup_id())
                        .map(|path| path.eq_ignore_ascii_case(&cgroup_path))
                        .unwrap_or(false)
                };
                if accept && tx.send(Ok(map_response(event))).await.is_err() {
                    // receiver is gone
                    break;
                }
            }
        });

        rx
    }
}

//TODO (jeroensoeters)  tests?!?!?
