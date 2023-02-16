/* -------------------------------------------------------------------------- *\
 *                      SPDX-License-Identifier: GPL-2.0                      *
 *                      SPDX-License-Identifier: MIT                          *
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
 * Dual Licensed: GNU GENERAL PUBLIC LICENSE 2.0                              *
 * Dual Licensed: MIT License                                                 *
 * Copyright 2023 The Aurae Authors (The Nivenly Foundation)                  *
\* -------------------------------------------------------------------------- */

#![no_std]
#![no_main]

use aurae_ebpf_shared::ForkedProcess;
use aya_bpf::macros::map;
use aya_bpf::macros::tracepoint;
use aya_bpf::maps::PerfEventArray;
use aya_bpf::programs::TracePointContext;

#[link_section = "license"]
#[used]
pub static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";

#[map(name = "FORKED_PROCESSES")]
static mut FORKED_PROCESSES: PerfEventArray<ForkedProcess> =
    PerfEventArray::<ForkedProcess>::with_max_entries(1024, 0);

const PARENT_PID_OFFSET: usize = 8;
const CHILD_PID_OFFSET: usize = 28;

#[tracepoint(name = "sched_process_fork")]
pub fn sched_process_fork(ctx: TracePointContext) -> u32 {
    match try_forked_process(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_forked_process(ctx: TracePointContext) -> Result<u32, u32> {
    let parent_pid: u32 = unsafe {
        match ctx.read_at(PARENT_PID_OFFSET) {
            Ok(s) => s,
            Err(errn) => return Err(errn as u32),
        }
    };

    let child_pid: u32 = unsafe {
        match ctx.read_at(CHILD_PID_OFFSET) {
            Ok(s) => s,
            Err(errn) => return Err(errn as u32),
        }
    };

    let s = ForkedProcess { parent_pid, child_pid };
    unsafe {
        FORKED_PROCESSES.output(&ctx, &s, 0);
    }
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
