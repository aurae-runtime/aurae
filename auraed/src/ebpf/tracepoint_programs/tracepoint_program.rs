use super::PerfEventBroadcast;
use aya::programs::ProgramError;
use aya::{
    maps::perf::AsyncPerfEventArray,
    programs::TracePoint,
    util::{nr_cpus, online_cpus},
    Bpf,
};
use bytes::BytesMut;
use log::trace;
use procfs::page_size;
use std::mem::size_of;
use tokio::sync::broadcast;
use tracing::{error, warn};

/// Size (in pages) for the circular per-CPU buffers that BPF perfbuf creates.
const PER_CPU_BUFFER_SIZE_IN_PAGES: usize = 2;

pub trait TracepointProgram<T: Clone + Send + 'static> {
    const PROGRAM_NAME: &'static str;
    const CATEGORY: &'static str;
    const EVENT: &'static str;
    const PERF_BUFFER: &'static str;

    fn load_and_attach(
        bpf: &mut Bpf,
    ) -> Result<PerfEventBroadcast<T>, anyhow::Error> {
        trace!("Loading eBPF program: {}", Self::PROGRAM_NAME);

        // Load the eBPF TracePoint program
        let program: &mut TracePoint = bpf
            .program_mut(Self::PROGRAM_NAME)
            .ok_or_else(|| anyhow::anyhow!("failed to get eBPF program"))?
            .try_into()?;

        // Load the program
        match program.load() {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyLoaded) => {
                warn!("Already loaded eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            other => other,
        }?;

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
        match program.attach(Self::CATEGORY, Self::EVENT) {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyAttached) => {
                warn!("Already attached eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            Err(e) => Err(e),
        }?;

        // Create the channel for broadcasting the events
        let (tx, _) = broadcast::channel(channel_capacity);

        // Open the BPF_PERF_EVENT_ARRAY BPF map that is used to send data from
        // kernel to userspace. This array contains the per-CPU buffers and is
        // indexed by CPU id.
        // https://libbpf.readthedocs.io/en/latest/api.html
        let mut perf_array =
            AsyncPerfEventArray::try_from(bpf.map_mut(Self::PERF_BUFFER)?)?;

        // Spawn a thread per CPU to listen for events from the kernel.
        for cpu_id in online_cpus()? {
            trace!("spawning task for cpu {cpu_id}");
            // Open the per-CPU buffer for the current CPU id
            let mut per_cpu_buffer =
                perf_array.open(cpu_id, Some(PER_CPU_BUFFER_SIZE_IN_PAGES))?;

            // Clone the sender of the event broadcast channel
            let per_cpu_tx = tx.clone();

            // Spawn the thread to listen on the per-CPU buffer
            let _ignored = tokio::spawn(async move {
                trace!("task for cpu {cpu_id} awaiting for events");

                // Allocate enough memory to drain the entire buffer
                // Note: using `vec!` macro will not result in a correct `Vec`
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
                            error!("fail to read events from per-cpu perf buffer, bailing out: {error}");
                            return;
                        }
                    };

                    if events.lost > 0 {
                        error!(
                            "buffer full, dropped {} perf events - this should never happen!",
                            events.lost
                        );
                    }

                    // If we don't have any receivers, there is no reason to send the signals to the channels.
                    // There is the possibility that a receiver subscribes while we are in the loop,
                    //   but this chooses performance over that possibility.
                    if per_cpu_tx.receiver_count() > 0 {
                        for buf in buffers.iter_mut().take(events.read) {
                            let ptr = buf.as_ptr() as *const T;
                            let signal = unsafe { ptr.read_unaligned() };
                            // send only errors if there are no receivers,
                            // so the return can be safely ignored;
                            // future sends may succeed
                            let _ = per_cpu_tx.send(signal);
                            // We don't clear buf for performance reasons (though it should be fast).
                            // Since we call `.take(events.read)` above, we shouldn't be re-reading old data
                            // buf.clear();
                        }
                    }
                }
            });
        }

        Ok(PerfEventBroadcast::new(tx))
    }
}
