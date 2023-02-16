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
    tracepoint_programs::{PerfEventBroadcast, TracepointProgram},
    BpfFile,
};
use aya::Bpf;

// This is critical to maintain the memory presence of the
// loaded bpf object.
// This specific BPF object needs to persist up to lib.rs such that
// the rest of the program can access this scope.
pub struct BpfHandle(Bpf);

impl BpfHandle {
    pub fn load() -> Result<Self, anyhow::Error> {
        let bpf = InstrumentTracepointSignalSignalGenerate::load()?;
        Ok(Self(bpf))
    }

    pub fn load_and_attach_tracepoint_program<TProgram, TEvent>(
        &mut self,
    ) -> Result<PerfEventBroadcast<TEvent>, anyhow::Error>
    where
        TProgram: TracepointProgram<TEvent>,
        TEvent: Clone + Send + 'static,
    {
        TProgram::load_and_attach(&mut self.0)
    }
}

struct InstrumentTracepointSignalSignalGenerate;
impl BpfFile for InstrumentTracepointSignalSignalGenerate {
    /// Definition of the Aurae eBPF probe to capture all generated (and valid)
    /// kernel signals at runtime.
    const OBJ_NAME: &'static str =
        "instrument-tracepoint-signal-signal-generate";
}
