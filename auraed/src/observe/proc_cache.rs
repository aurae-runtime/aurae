/* -------------------------------------------------------------------------- *\
 *          Apache 2.0 License Copyright © 2022-2023 The Aurae Authors        *
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
// TODO(jeroen) remove once this is fully integrated
#![allow(dead_code)]

use crate::ebpf::tracepoint::PerfEventBroadcast;
use aurae_ebpf_shared::{ForkedProcess, ProcessExit};
use std::time::SystemTime;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

#[cfg(not(test))]
pub fn now() -> SystemTime {
    SystemTime::now()
}
#[cfg(test)]
pub fn now() -> SystemTime {
    mock_time::now()
}

pub trait ProcessInfo {
    fn get_nspid(&self, pid: i32) -> Option<i32>;
}

pub(crate) struct ProcfsProcessInfo {}

impl ProcessInfo for ProcfsProcessInfo {
    fn get_nspid(&self, pid: i32) -> Option<i32> {
        procfs::process::Process::new(pid)
            .and_then(|p| p.status())
            .ok()
            .and_then(|s| s.nspid)
            .and_then(|nspid| nspid.last().copied())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Eviction {
    pid: i32,
    evict_at: SystemTime,
}

#[derive(Debug)]
pub struct ProcCache {
    cache: Arc<Mutex<HashMap<i32, i32>>>,
    evict_after: Duration,
    eviction_queue: Arc<Mutex<VecDeque<Eviction>>>,
}

static PID_MAX: usize = 4194304;

impl ProcCache {
    pub fn new(
        evict_after: Duration,
        process_fork_events: PerfEventBroadcast<ForkedProcess>,
        process_exit_events: PerfEventBroadcast<ProcessExit>,
        proc_info: impl ProcessInfo + Send + 'static + Sync,
    ) -> Self {
        let res = Self {
            cache: Arc::new(Mutex::new(HashMap::with_capacity(PID_MAX))),
            evict_after,
            eviction_queue: Arc::new(Mutex::new(VecDeque::with_capacity(
                PID_MAX,
            ))),
        };

        let mut process_fork_rx = process_fork_events.subscribe();
        let cache_for_fork_event_processing = res.cache.clone();
        let _ignored = tokio::spawn(async move {
            while let Ok(e) = process_fork_rx.recv().await {
                if let Some(nspid) = proc_info.get_nspid(e.child_pid) {
                    let mut guard =
                        cache_for_fork_event_processing.lock().await;
                    let _ = guard.insert(e.child_pid, nspid);
                }
            }
        });

        let mut process_exit_rx = process_exit_events.subscribe();
        let eviction_queue_for_exit_event_processing =
            res.eviction_queue.clone();
        let _ignored = tokio::spawn(async move {
            while let Ok(e) = process_exit_rx.recv().await {
                let mut guard =
                    eviction_queue_for_exit_event_processing.lock().await;
                guard.push_back(Eviction {
                    pid: e.pid,
                    evict_at: now().checked_add(evict_after).expect(
                        "SystemTime overflow, something has gone very wrong!",
                    ),
                })
            }
        });

        res
    }

    pub async fn get(&self, pid: i32) -> Option<i32> {
        self.evict_expired().await;

        let guard = self.cache.lock().await;
        guard.get(&pid).copied()
    }

    async fn evict_expired(&self) {
        let now = now();
        let mut queue_guard = self.eviction_queue.lock().await;
        let mut evict = Vec::with_capacity(64);
        while let Some(_v) = queue_guard.front().filter(|v| v.evict_at <= now) {
            evict.push(queue_guard.pop_front().expect(
                "the let Some(v) binding guarantees that thsi option is set",
            ))
        }
        drop(queue_guard);
        let mut cache_guard = self.cache.lock().await;
        for e in evict {
            _ = cache_guard.remove(&e.pid);
        }
    }

    #[cfg(test)]
    async fn eviction_queue(&self) -> VecDeque<Eviction> {
        //let mut queue = VecDeqe::new();
        let guard = self.eviction_queue.lock().await;
        guard.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ebpf::tracepoint::PerfEventBroadcast;
    use crate::observe::proc_cache::ForkedProcess;
    use serial_test::serial;
    use tokio::sync::broadcast::{channel, Sender};

    struct TestProcessInfo {
        nspid_lookup: HashMap<i32, i32>,
    }

    impl TestProcessInfo {
        fn new(test_data: Vec<(i32, i32)>) -> Self {
            let mut nspid_lookup = HashMap::new();
            for (pid, nspid) in test_data {
                _ = nspid_lookup.insert(pid, nspid);
            }
            Self { nspid_lookup }
        }
    }

    impl ProcessInfo for TestProcessInfo {
        fn get_nspid(&self, pid: i32) -> Option<i32> {
            self.nspid_lookup.get(&pid).copied()
        }
    }

    #[tokio::test]
    async fn must_returm_none_for_non_existing_process() {
        let (cache, _, _) = cache_for_testing(Duration::from_secs(5), vec![]);

        assert_eq!(cache.get(123).await, None);
    }

    #[tokio::test]
    #[serial] // Needs to run in isolation because of the mocked `SystemTime`
    async fn must_create_cache_entry_for_a_new_processes() {
        let (cache, fork_tx, _) =
            cache_for_testing(Duration::from_secs(5), vec![(42, 2)]);

        _ = fork_tx
            .send(ForkedProcess { parent_pid: 1, child_pid: 42 })
            .expect("error sending msg");

        //TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(cache.get(42).await, Some(2));
    }

    #[tokio::test]
    #[serial] // Needs to run in isolation because of the mocked `SystemTime`
    async fn must_mark_entry_for_eviction_when_a_process_exits() {
        mock_time::reset();
        let (cache, fork_tx, exit_tx) = cache_for_testing(
            Duration::from_secs(5),
            vec![(42, 2), (43, 3), (44, 4)],
        );

        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 42 });
        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 43 });
        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 44 });

        _ = exit_tx.send(ProcessExit { pid: 42 });

        // TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        mock_time::advance_time(Duration::from_secs(5));

        _ = exit_tx.send(ProcessExit { pid: 44 });

        //TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(
            cache.eviction_queue().await,
            vec![
                Eviction { pid: 42, evict_at: seconds_after_unix_epoch(5) },
                Eviction { pid: 44, evict_at: seconds_after_unix_epoch(10) }
            ],
        );
    }

    #[tokio::test]
    #[serial] // Needs to run in isolation because of the mocked `SystemTime`
    async fn must_evict_expired_entries_from_cache_on_get() {
        mock_time::reset();
        let (cache, fork_tx, exit_tx) = cache_for_testing(
            Duration::from_secs(5),
            vec![(42, 2), (43, 3), (44, 4), (45, 5)],
        );

        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 42 });
        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 43 });
        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 44 });
        _ = fork_tx.send(ForkedProcess { parent_pid: 1, child_pid: 45 });

        _ = exit_tx.send(ProcessExit { pid: 42 });

        //TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        mock_time::advance_time(Duration::from_secs(2));

        _ = exit_tx.send(ProcessExit { pid: 44 });

        //TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        mock_time::advance_time(Duration::from_secs(5));

        _ = exit_tx.send(ProcessExit { pid: 45 });

        //TODO (jeroensoeters) change to polling
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(
            cache.eviction_queue().await,
            vec![
                Eviction { pid: 42, evict_at: seconds_after_unix_epoch(5) }, // T(event) = 0 -> T(evict) = 5
                Eviction { pid: 44, evict_at: seconds_after_unix_epoch(7) }, // T(event) = 2 -> T(evict) = 7
                Eviction { pid: 45, evict_at: seconds_after_unix_epoch(12) } // T(event) = 7 -> T(evict) = 12
            ],
        );

        assert_eq!(cache.get(42).await, None); // expired
        assert_eq!(cache.get(43).await, Some(3)); // never exited
        assert_eq!(cache.get(44).await, None); // expired
        assert_eq!(cache.get(45).await, Some(5)); // still queued for eviction
    }

    fn seconds_after_unix_epoch(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(seconds))
            .unwrap()
    }

    fn cache_for_testing(
        expire_after: Duration,
        test_data: Vec<(i32, i32)>,
    ) -> (ProcCache, Sender<ForkedProcess>, Sender<ProcessExit>) {
        let (fork_tx, _fork_rx) = channel(4);
        let fork_broadcaster = PerfEventBroadcast::new(fork_tx.clone());
        let (exit_tx, _exit_rx) = channel::<ProcessExit>(4);
        let exit_broadcaster = PerfEventBroadcast::new(exit_tx.clone());

        let test_proc_info = TestProcessInfo::new(test_data);

        let cache = ProcCache::new(
            expire_after,
            fork_broadcaster,
            exit_broadcaster,
            test_proc_info,
        );

        (cache, fork_tx, exit_tx)
    }
}

#[cfg(test)]
mod mock_time {
    use once_cell::sync::OnceCell;
    use std::sync::Mutex;
    use std::time::{Duration, SystemTime};

    pub static TIME: OnceCell<Mutex<SystemTime>> = OnceCell::new();

    pub fn now() -> SystemTime {
        *TIME.get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH)).lock().unwrap()
    }

    pub fn advance_time(d: Duration) {
        let mut guard = TIME
            .get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH))
            .lock()
            .unwrap();
        *guard = guard.checked_add(d).unwrap();
    }

    pub fn reset() {
        let mut guard = TIME
            .get_or_init(|| Mutex::new(SystemTime::UNIX_EPOCH))
            .lock()
            .unwrap();
        *guard = SystemTime::UNIX_EPOCH;
    }
}
