use aurae_ebpf_shared::Signal;
use bytes::BytesMut;
use std::mem::size_of;
use tokio::sync::broadcast;
use tracing::{trace, warn};

use crate::ebpf::perf_event_listener::PerfEventListener;

use aya::{
    include_bytes_aligned, maps::perf::AsyncPerfEventArray,
    programs::TracePoint, util::online_cpus, Bpf,
};

pub struct BpfLoader {
    bpf: Bpf,
}

impl BpfLoader {
    pub fn new() -> Result<Self, anyhow::Error> {
        #[cfg(debug_assertions)]
        let bpf = Bpf::load(include_bytes_aligned!(
            "../../../aurae-ebpf/target/bpfel-unknown-none/debug/aurae-ebpf"
        ))?;

        #[cfg(not(debug_assertions))]
        let bpf = Bpf::load(include_bytes_aligned!(
            "../../../aurae-ebpf/target/bpfel-unknown-none/release/aurae-ebpf"
        ))?;

        Ok(Self { bpf })
    }

    pub fn load_signals_tracepoint(&mut self) -> Result<PerfEventListener<Signal>, anyhow::Error> {
        self.load_tracepoint::<Signal>(
            "signals",
            "signal",
            "signal_generate",
            "SIGNALS",
        )
    }

    fn load_tracepoint<T: Clone + Send + 'static>(
        &mut self,
        prog_name: &str,
        category: &str,
        event: &str,
        perf_buffer: &str,
    ) -> Result<PerfEventListener<T>, anyhow::Error> {
        // Load the eBPF Tracepoint program
        let program: &mut TracePoint = self
            .bpf
            .program_mut(prog_name)
            .expect("failed to load tracepoint")
            .try_into()?;
        program.load()?;

        // Attach to kernel trace event
        let _ = program.attach(category, event)?;

        // Spawn a thread per CPU to listen for events from the kernel. Each thread has its own perf event buffer.
        let (tx, _) = broadcast::channel(100);
        let signal_struct_size: usize = size_of::<T>();
        let mut perf_array =
            AsyncPerfEventArray::try_from(self.bpf.map_mut(perf_buffer)?)?;
        for cpu_id in online_cpus()? {
            trace!("spawning task for cpu {}", cpu_id);
            let mut per_cpu_buffer = perf_array.open(cpu_id, None)?;
            let per_cpu_tx = tx.clone();
            let _ = tokio::spawn(async move {
                trace!("task for cpu awaiting for events {}", cpu_id);
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
                                let errstr = format!("{}", err);
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
