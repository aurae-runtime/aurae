#![no_std]
#![no_main]

use aya_bpf::{macros::tracepoint, programs::TracePointContext};
use aya_log_ebpf::info;

#[tracepoint(name = "signals")]
pub fn signals(ctx: TracePointContext) -> u32 {
    match try_signals(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_signals(ctx: TracePointContext) -> Result<u32, u32> {
    info!(&ctx, "tracepoint signal_generate called");
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
