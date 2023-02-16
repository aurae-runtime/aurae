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
use aya_bpf::helpers;
use aya_bpf::macros::fentry;
use aya_bpf::macros::map;
use aya_bpf::maps::PerfEventArray;
use aya_bpf::programs::TracePointContext;

#[link_section = "license"]
#[used]
pub static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";

#[map(name = "PROCESS_EXITS")]
static mut PROCESS_EXITS: PerfEventArray<u32> =
    PerfEventArray::<u32>::with_max_entries(1024, 0);

const PARENT_PID_OFFSET: usize = 8;
const CHILD_PID_OFFSET: usize = 28;

#[fentry(name = "fentry_taskstats_exit")]
pub fn fentry_taskstats_exit(ctx: TracePointContext) -> u32 {
    let pid = helpers::bpf_get_current_pid_tgid();

    unsafe {
        PROCESS_EXITS.output(&ctx, pid, 0);
    }

    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
