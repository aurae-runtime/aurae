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

use super::{SocketStream, SystemRuntime, SystemRuntimeError};
use crate::init::{
    fs::MountSpec, logging, network, power::spawn_thread_power_button_listener,
    system_runtimes::create_tcp_socket_stream, BANNER,
};
use std::{net::SocketAddr, path::Path};
use tonic::async_trait;
use tracing::{error, info, trace};

const POWER_BUTTON_DEVICE: &str = "/dev/input/event0";
const DEFAULT_NETWORK_SOCKET_ADDR: &str = "[::]:8080";

pub(crate) struct Pid1SystemRuntime;

impl Pid1SystemRuntime {
    fn spawn_system_runtime_threads(&self) {
        // ---- MAIN DAEMON THREAD POOL ----
        // TODO: https://github.com/aurae-runtime/auraed/issues/33
        match spawn_thread_power_button_listener(Path::new(POWER_BUTTON_DEVICE))
        {
            Ok(_) => {
                info!("Spawned power button device listener");
            }
            Err(e) => {
                error!(
                    "Failed to spawn power button device listener. Error={e}"
                );
            }
        }

        // ---- MAIN DAEMON THREAD POOL ----
    }
}

#[async_trait]
impl SystemRuntime for Pid1SystemRuntime {
    // Executing as PID 1 context
    async fn init(
        self,
        verbose: bool,
        socket_address: Option<String>,
    ) -> Result<SocketStream, SystemRuntimeError> {
        println!("{BANNER}");

        // Initialize the PID 1 logger
        logging::init(verbose, false)?;
        info!("Running as pid 1");
        trace!("Configure filesystem");

        // NOTE: THESE TODOS WERE ALL HERE, BUT...
        //       if you are here, you are auraed is true pid 1
        //       Container -> use container_system_runtime.rs
        //       Cell -> use cell_system_runtime.rs

        // TODO We need to determine how we want to handle mountings these filesystems.
        // TODO From within the context of a container (cgroup trailing / in cgroup namespace)
        // TODO We likely to do not need to mount these filesystems.
        // TODO Do we want to have a way to "try" these mounts and continue without erroring?

        MountSpec { source: None, target: "/sys", fstype: Some("sysfs") }
            .mount()?;

        MountSpec {
            source: Some("proc"),
            target: "/proc",
            fstype: Some("proc"),
        }
        .mount()?;

        MountSpec {
            source: Some("debugfs"),
            target: "/sys/kernel/debug",
            fstype: Some("debugfs"),
        }
        .mount()?;

        trace!("Configure network");
        // show_dir("/sys/class/net/", false); // Show available network interfaces
        let network = network::Network::connect()?;
        network.init().await?;
        network.show_network_info().await;

        // TODO: do we need to create an interface and address for socket_address?

        self.spawn_system_runtime_threads();

        trace!("init of auraed as pid1 done");

        let socket_addr = socket_address
            .unwrap_or_else(|| DEFAULT_NETWORK_SOCKET_ADDR.into())
            .parse::<SocketAddr>()?;
        create_tcp_socket_stream(socket_addr).await
    }
}