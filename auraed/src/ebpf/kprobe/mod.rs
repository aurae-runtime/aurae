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
use super::{bpf_file::BpfFile, perf_buffer_reader::PerfBufferReader};
use aurae_ebpf_shared::ProcessExit;
pub use kprobe_program::KProbeProgram;

mod kprobe_program;

pub struct TaskstatsExitKProbeProgram;

impl KProbeProgram<ProcessExit> for TaskstatsExitKProbeProgram {
    const PROGRAM_NAME: &'static str = "kprobe_taskstats_exit";
    const FUNCTION_NAME: &'static str = "taskstats_exit";
    const PERF_BUFFER: &'static str = "PROCESS_EXITS";
}

impl BpfFile for TaskstatsExitKProbeProgram {
    /// Definition of the Aurae eBPF probe to capture all generated (and valid)
    /// kernel signals at runtime.
    const OBJ_NAME: &'static str = "instrument-kprobe-taskstats-exit";
}

impl PerfBufferReader<ProcessExit> for TaskstatsExitKProbeProgram {}