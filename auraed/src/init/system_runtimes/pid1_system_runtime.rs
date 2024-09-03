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
    fs::{FsError, MountSpec, CGROUP_MNT_FLAGS, CHMOD_0755, COMMON_MNT_FLAGS},
    logging, network,
    power::spawn_thread_power_button_listener,
    system_runtimes::create_tcp_socket_stream,
    BANNER,
};
use nix::{
    mount::MsFlags,
    unistd::{mkdir, symlinkat},
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

        mkdir("/dev/pts", *CHMOD_0755).map_err(FsError::FileCreationFailure)?;
        MountSpec {
            source: Some("devpts"),
            target: "/dev/pts",
            fstype: Some("devpts"),
            flags: MsFlags::MS_NOEXEC
                | MsFlags::MS_NOSUID
                | MsFlags::MS_NOATIME,
            data: Some("mode=0620,gid=5,ptmxmode=666"),
        }
        .mount()?;

        MountSpec {
            source: Some("sysfs"),
            target: "/sys",
            fstype: Some("sysfs"),
            flags: *COMMON_MNT_FLAGS,
            data: None,
        }
        .mount()?;

        MountSpec {
            source: Some("proc"),
            target: "/proc",
            fstype: Some("proc"),
            flags: *COMMON_MNT_FLAGS,
            data: None,
        }
        .mount()?;

        MountSpec {
            source: Some("run"),
            target: "/run",
            fstype: Some("tmpfs"),
            flags: MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
            data: Some("mode=0755"),
        }
        .mount()?;

        symlinkat("/proc/self/fd", None, "/dev/fd")
            .map_err(FsError::FileCreationFailure)?;
        symlinkat("/proc/self/fd/0", None, "/dev/stdin")
            .map_err(FsError::FileCreationFailure)?;
        symlinkat("/proc/self/fd/1", None, "/dev/stdout")
            .map_err(FsError::FileCreationFailure)?;
        symlinkat("/proc/self/fd/2", None, "/dev/stderr")
            .map_err(FsError::FileCreationFailure)?;

        MountSpec {
            source: Some("cgroup2"),
            target: "/sys/fs/cgroup",
            fstype: Some("cgroup2"),
            flags: *CGROUP_MNT_FLAGS,
            data: None,
        }
        .mount()?;

        MountSpec {
            source: Some("debugfs"),
            target: "/sys/kernel/debug",
            fstype: Some("debugfs"),
            flags: *COMMON_MNT_FLAGS,
            data: None,
        }
        .mount()?;

        trace!("Configure network");

        const DEFAULT_NET_DEV: &str = "eth0";
        const DEFAULT_NET_DEV_IPV6: &str = "fe80::2";
        const DEFAULT_NET_DEV_IPV6_GATEWAY: &str = "fe80::1";
        const DEFAULT_NET_DEV_IPV6_SUBNET: &str = "/64";

        let network = network::Network::connect()?;
        network
            .init(&network::Config {
                device: DEFAULT_NET_DEV.to_owned(),
                address: DEFAULT_NET_DEV_IPV6.to_owned(),
                gateway: DEFAULT_NET_DEV_IPV6_GATEWAY.to_owned(),
                subnet: DEFAULT_NET_DEV_IPV6_SUBNET.to_owned(),
            })
            .await?;
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
