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

use super::{
    kprobe::KProbeProgram, perf_buffer_reader::PerfBufferReader,
    perf_event_broadcast::PerfEventBroadcast, tracepoint::TracepointProgram,
    BpfFile,
};

use aya::Bpf;
use tracing::warn;

// This is critical to maintain the memory presence of the
// loaded bpf object.
// This specific BPF object needs to persist up to lib.rs such that
// the rest of the program can access this scope.
pub struct BpfContext(Vec<Bpf>);

impl BpfContext {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn load_and_attach_tracepoint_program<TProgram, TEvent>(
        &mut self,
    ) -> Result<PerfEventBroadcast<TEvent>, anyhow::Error>
    where
        TProgram:
            BpfFile + TracepointProgram<TEvent> + PerfBufferReader<TEvent>,
        TEvent: Clone + Send + 'static,
    {
        match TProgram::load() {
            Ok(mut bpf_handle) => {
                TProgram::load_and_attach(&mut bpf_handle)?;
                let perf_events = TProgram::read_from_perf_buffer(
                    &mut bpf_handle,
                    TProgram::PERF_BUFFER,
                );
                self.0.push(bpf_handle);
                perf_events
            }
            Err(e) => {
                warn!(
                    "Error loading tracepoint program {}: {}",
                    TProgram::PROGRAM_NAME,
                    e
                );
                Err(e.into())
            }
        }
    }

    pub fn load_and_attach_kprobe_program<TProgram, TEvent>(
        &mut self,
    ) -> Result<PerfEventBroadcast<TEvent>, anyhow::Error>
    where
        TProgram: BpfFile + KProbeProgram<TEvent> + PerfBufferReader<TEvent>,
        TEvent: Clone + Send + 'static + std::fmt::Debug,
    {
        match TProgram::load() {
            Ok(mut bpf_handle) => {
                TProgram::load_and_attach(&mut bpf_handle)?;
                let perf_events = TProgram::read_from_perf_buffer(
                    &mut bpf_handle,
                    TProgram::PERF_BUFFER,
                );
                self.0.push(bpf_handle);
                perf_events
            }
            Err(e) => {
                warn!(
                    "Error loading kprobe program {}: {}",
                    TProgram::PROGRAM_NAME,
                    e
                );
                Err(e.into())
            }
        }
    }
}
