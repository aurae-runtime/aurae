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

use crate::ebpf::perf_event_listener::PerfEventListener;
use crate::AURAE_LIBRARY_DIR;
use aurae_ebpf_shared::Signal;
use aya::util::nr_cpus;
use aya::{
    maps::perf::AsyncPerfEventArray, programs::TracePoint, util::online_cpus,
    Bpf,
};
use bytes::BytesMut;
use log::trace;
use procfs::page_size;
use std::mem::size_of;
use tokio::sync::broadcast;
use tracing::error;

pub struct BpfLoader {
    // The "bpf_scope" is critical to maintain the memory presence of the
    // loaded bpf object.
    // This specific BPF object needs to persist up to lib.rs such that
    // the rest of the program can access this scope.
    bpf_scopes: Vec<Bpf>,
}

/// Definition of the Aurae eBPF probe to capture all generated (and valid)
/// kernel signals at runtime.
const INSTRUMENT_TRACEPOINT_SIGNAL_SIGNAL_GENERATE: &str =
    "instrument-tracepoint-signal-signal-generate";

/// Size (in pages) for the circular per-CPU buffers that BPF perfbuf creates.
const PER_CPU_BUFFER_SIZE_IN_PAGES: usize = 2;

impl BpfLoader {
    pub fn new() -> Self {
        Self { bpf_scopes: vec![] }
    }

    pub fn read_and_load_tracepoint_signal_signal_generate(
        &mut self,
    ) -> Result<PerfEventListener<Signal>, anyhow::Error> {
        self.load_perf_event_program::<Signal>(
            INSTRUMENT_TRACEPOINT_SIGNAL_SIGNAL_GENERATE,
            "signals",
            "signal",
            "signal_generate",
            "SIGNALS",
        )
    }

    /// Load a "PerfEvent" BPF program at runtime given an ELF object and
    /// program configuration.
    fn load_perf_event_program<T: Clone + Send + 'static>(
        &mut self,
        aurae_obj_name: &str,
        prog_name: &str,
        category: &str,
        event: &str,
        perf_buffer: &str,
    ) -> Result<PerfEventListener<T>, anyhow::Error> {
        trace!("Loading eBPF program: {}", aurae_obj_name);
        let mut bpf_object = Bpf::load_file(format!(
            "{AURAE_LIBRARY_DIR}/ebpf/{aurae_obj_name}",
        ))?;

        // Load the eBPF Tracepoint program
        let program: &mut TracePoint = bpf_object
            .program_mut(prog_name)
            .expect("failed to load tracepoint")
            .try_into()?;

        // Load the program
        program.load()?;

        // Query the number of CPUs on the host
        let num_cpus = nr_cpus()?;

        // Query the page size on the host
        let page_size = page_size()?;

        // Get the size of the event payload
        let event_struct_size: usize = size_of::<T>();

        // Calculate the capacity of the per-CPU buffers based on the size of
        // the event
        let per_cpu_buffer_capacity = (PER_CPU_BUFFER_SIZE_IN_PAGES
            * page_size as usize)
            / event_struct_size;

        // Set the capacity of the channel to the combined capacity of all the
        // per-CPU buffers
        let channel_capacity = per_cpu_buffer_capacity * num_cpus;

        // Attach to kernel trace event
        let _ = program.attach(category, event)?;

        // Create the channel for braodcasting the events
        let (tx, _) = broadcast::channel(channel_capacity);

        // Open the BPF_PERF_EVENT_ARRAY BPF map that is used to send data from
        // kernel to userspace. This array contains the per-CPU buffers and is
        // indexed by CPU id.
        // https://libbpf.readthedocs.io/en/latest/api.html
        let mut perf_array =
            AsyncPerfEventArray::try_from(bpf_object.map_mut(perf_buffer)?)?;

        // Spawn a thread per CPU to listen for events from the kernel.
        for cpu_id in online_cpus()? {
            trace!("spawning task for cpu {}", cpu_id);
            // Open the per-CPU buffer for the current CPU id
            let mut per_cpu_buffer =
                perf_array.open(cpu_id, Some(PER_CPU_BUFFER_SIZE_IN_PAGES))?;

            // Clone the sender of the event broadcast channel
            let per_cpu_tx = tx.clone();

            // Spawn the thread to listen on the per-CPU buffer
            let _ignored = tokio::spawn(async move {
                trace!("task for cpu awaiting for events {}", cpu_id);

                // Allocate enough memory to drain the entire buffer
                let mut buffers = (0..per_cpu_buffer_capacity)
                    .map(|_| BytesMut::with_capacity(event_struct_size))
                    .collect::<Vec<_>>();

                // Start polling the per-CPU buffer for events
                loop {
                    let events = match per_cpu_buffer
                        .read_events(&mut buffers)
                        .await
                    {
                        Ok(events) => events,
                        Err(error) => {
                            error!("fail to read events from per-cpu perf buffer, bailing out: {}", error);
                            return;
                        }
                    };

                    if events.lost > 0 {
                        error!(
                            "buffer full, dropped {} perf events - this should never happen!",
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
                                    error!(
                                        "failed to broadcast perf event: {}",
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
            });
        }

        // Append to bpf_scopes
        self.bpf_scopes.push(bpf_object);

        Ok(PerfEventListener::new(tx))
    }
}
