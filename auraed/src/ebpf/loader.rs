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

use aurae_ebpf_shared::Signal;
use bytes::BytesMut;
use std::mem::size_of;
use tokio::sync::broadcast;
use tracing::{trace, warn};

use crate::ebpf::perf_event_listener::PerfEventListener;

use crate::AURAE_LIBRARY_DIR;
use aya::util::nr_cpus;
use aya::{
    maps::perf::AsyncPerfEventArray, programs::TracePoint, util::online_cpus,
    Bpf,
};
use log::info;

pub struct BpfLoader {}

/// Definition of the Aurae eBPF probe to capture all generated (and valid)
/// kernel signals at runtime.
const INSTRUMENT_TRACEPOINT_SIGNAL_SIGNAL_GENERATE: &str =
    "instrument-tracepoint-signal-signal-generate";

/// Definition of the channel capacity for the perf event buffer fro ALL CPUs.
// TODO: We should consider some basic math here. Perhaps 1024 * Available CPUs (Relevant for nested auraed in cgroups)
const CHANNEL_CAPACITY: usize = 1024;

impl BpfLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_and_load_tracepoint_signal_signal_generate(
        &mut self,
    ) -> Result<PerfEventListener<Signal>, anyhow::Error> {
        info!(
            "Loading eBPF program: {}",
            INSTRUMENT_TRACEPOINT_SIGNAL_SIGNAL_GENERATE
        );
        let object = Bpf::load_file(format!(
            "{}/ebpf/{}",
            AURAE_LIBRARY_DIR, INSTRUMENT_TRACEPOINT_SIGNAL_SIGNAL_GENERATE
        ))?;

        self.load_perf_event_program::<Signal>(
            object,
            "signals",
            "signal",
            "signal_generate",
            "SIGNALS",
        )
    }

    /// Load a "PerfEvent" BPF program at runtime given an ELF object and
    /// program configuration.
    fn load_perf_event_program<T: Clone + Send + 'static>(
        &self,
        mut bpf_object: Bpf,
        prog_name: &str,
        category: &str,
        event: &str,
        perf_buffer: &str,
    ) -> Result<PerfEventListener<T>, anyhow::Error> {
        // Load the eBPF Tracepoint program
        let program: &mut TracePoint = bpf_object
            .program_mut(prog_name)
            .expect("failed to load tracepoint")
            .try_into()?;

        // Load the program
        program.load()?;

        // Attach to kernel trace event
        let _ = program.attach(category, event)?;

        // Spawn a thread per CPU to listen for events from the kernel. Each thread has its own perf event buffer.
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let signal_struct_size: usize = size_of::<T>();
        let mut perf_array =
            AsyncPerfEventArray::try_from(bpf_object.map_mut(perf_buffer)?)?;

        let _num_cpus = nr_cpus()?;
        for cpu_id in online_cpus()? {
            trace!("spawning task for cpu {}", cpu_id);
            let mut per_cpu_buffer = perf_array.open(cpu_id, None)?;
            let per_cpu_tx = tx.clone();
            let _ignored = tokio::spawn(async move {
                trace!("task for cpu awaiting for events {}", cpu_id);
                // Calculate the capacity of events per CPU
                let _buffer_max = _num_cpus * 64;
                let mut buffers = (0..100)
                    .map(|_| BytesMut::with_capacity(signal_struct_size))
                    .collect::<Vec<_>>();

                loop {
                    let events = match per_cpu_buffer
                        .read_events(&mut buffers)
                        .await
                    {
                        Ok(events) => events,
                        Err(error) => {
                            warn!("fail to read events from the perf, bailing out: {}", error);
                            return;
                        }
                    };

                    if events.lost > 0 {
                        warn!(
                            "queues are getting full, lost {} perf events",
                            events.lost
                        );
                    }

                    for buf in buffers.iter_mut().take(events.read) {
                        let ptr = buf.as_ptr() as *const T;
                        let signal = unsafe { ptr.read_unaligned() };
                        match per_cpu_tx.send(signal) {
                            Ok(_) => continue,
                            Err(err) => {
                                // if no one is listening the error returned. XXX find a
                                // better way of handling this.
                                let errstr = format!("{err}");
                                if !errstr.contains("channel closed") {
                                    warn!(
                                        "failed to send perf event internally: {}",
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
            });
        }

        Ok(PerfEventListener::new(tx))
    }
}
