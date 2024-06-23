# ---------------------------------------------------------------------------- #
#                +--------------------------------------------+                #
#                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |                #
#                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |                #
#                |  ███████║██║   ██║██████╔╝███████║█████╗   |                #
#                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |                #
#                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |                #
#                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |                #
#                +--------------------------------------------+                #
#                                                                              #
#                         Distributed Systems Runtime                          #
# ---------------------------------------------------------------------------- #
# Copyright 2022 - 2024, the aurae contributors
# SPDX-License-Identifier: Apache-2.0

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

use super::bpf_file::BpfFile;
use super::perf_buffer_reader::PerfBufferReader;
pub use crate::ebpf::perf_event_broadcast::PerfEventBroadcast;
use aurae_ebpf_shared::{ForkedProcess, Signal};
pub use tracepoint_program::TracepointProgram;

mod tracepoint_program;

pub struct SignalSignalGenerateTracepointProgram;

impl TracepointProgram<Signal> for SignalSignalGenerateTracepointProgram {
    const PROGRAM_NAME: &'static str = "signal_signal_generate";
    const CATEGORY: &'static str = "signal";
    const EVENT: &'static str = "signal_generate";
    const PERF_BUFFER: &'static str = "SIGNALS";
}

impl BpfFile for SignalSignalGenerateTracepointProgram {
    /// Definition of the Aurae eBPF probe to capture all generated (and valid)
    /// kernel signals at runtime.
    const OBJ_NAME: &'static str =
        "instrument-tracepoint-signal-signal-generate";
}

impl PerfBufferReader<Signal> for SignalSignalGenerateTracepointProgram {}

pub struct SchedProcessForkTracepointProgram;

impl TracepointProgram<ForkedProcess> for SchedProcessForkTracepointProgram {
    const PROGRAM_NAME: &'static str = "sched_process_fork";
    const CATEGORY: &'static str = "sched";
    const EVENT: &'static str = "sched_process_fork";
    const PERF_BUFFER: &'static str = "FORKED_PROCESSES";
}

impl BpfFile for SchedProcessForkTracepointProgram {
    /// Definition of the Aurae eBPF probe to capture all generated (and valid)
    /// kernel signals at runtime.
    const OBJ_NAME: &'static str =
        "instrument-tracepoint-sched-sched-process-fork";
}

impl PerfBufferReader<ForkedProcess> for SchedProcessForkTracepointProgram {}
