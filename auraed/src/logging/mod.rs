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

//! Internal logging system for Auraed and all spawned Executables, Containers
//! and Instances.

use std::time::SystemTime;

/// Abstraction Layer for one log generating entity
/// LogChannel provides channels between Log producers and log consumers
pub mod log_channel;

/// `MakeWriter` adapter that pipes formatted `tracing` events into a `LogChannel` broadcast.
pub mod broadcast_writer;

/// Get UNIX timestamp in seconds for logging
pub fn get_timestamp_sec() -> i64 {
    let unix_ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System Clock went backwards");

    unix_ts.as_secs() as i64
}
