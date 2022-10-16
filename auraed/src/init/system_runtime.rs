use crate::init::network::{
    add_address, add_route_v6, set_link_up, show_network_info,
};
use crate::init::power::spawn_thread_power_button_listener;
use crate::init::{fs, logging, InitError, BANNER};
use anyhow::anyhow;
use ipnetwork::{IpNetwork, Ipv6Network};
use log::{error, info, trace, Level};
use netlink_packet_route::RtnlMessage;
use rtnetlink::new_connection;
use rtnetlink::proto::Connection;
use std::path::Path;
use tonic::async_trait;

const LOOPBACK_DEV: &str = "lo";

const LOOPBACK_IPV6: &str = "::1";
const LOOPBACK_IPV6_SUBNET: &str = "/128";

const LOOPBACK_IPV4: &str = "127.0.0.1";
const LOOPBACK_IPV4_SUBNET: &str = "/8";

const DEFAULT_NET_DEV: &str = "eth0";
const DEFAULT_NET_DEV_IPV6: &str = "fe80::2";
const DEFAULT_NET_DEV_IPV6_SUBNET: &str = "/64";

const POWER_BUTTON_DEVICE: &str = "/dev/input/event0";

#[async_trait]
pub(crate) trait SystemRuntime {
    async fn init(self, logger_level: Level) -> Result<(), InitError>;
}

pub(crate) struct Pid1SystemRuntime;

impl Pid1SystemRuntime {
    async fn init_network(
        &self,
        connection: Connection<RtnlMessage>,
        handle: &rtnetlink::Handle,
    ) {
        tokio::spawn(connection);

        trace!("configure {}", LOOPBACK_DEV);
        match self.configure_loopback(handle).await {
            Ok(_) => {
                info!("Successfully configured {}", LOOPBACK_DEV);
            }
            Err(e) => {
                error!("Failed to setup loopback device. Error={}", e);
            }
        }

        trace!("configure {}", DEFAULT_NET_DEV);

        match self.configure_nic(handle).await {
            Ok(_) => {
                info!("Successfully configured {}", DEFAULT_NET_DEV);
            }
            Err(e) => {
                error!(
                    "Failed to configure NIC {}. Error={}",
                    DEFAULT_NET_DEV, e
                );
            }
        }

        show_network_info(handle).await;
    }

    async fn configure_loopback(
        &self,
        handle: &rtnetlink::Handle,
    ) -> anyhow::Result<()> {
        if let Ok(ipv6) = format!("{}{}", LOOPBACK_IPV6, LOOPBACK_IPV6_SUBNET)
            .parse::<IpNetwork>()
        {
            if let Err(e) = add_address(LOOPBACK_DEV, ipv6, handle).await {
                return Err(anyhow!("Failed to add ipv6 address to loopback device {}. Error={}", LOOPBACK_DEV, e));
            };
        }

        if let Ok(ipv4) = format!("{}{}", LOOPBACK_IPV4, LOOPBACK_IPV4_SUBNET)
            .parse::<IpNetwork>()
        {
            if let Err(e) = add_address(LOOPBACK_DEV, ipv4, handle).await {
                return Err(anyhow!("Failed to add ipv4 address to loopback device {}. Error={}", LOOPBACK_DEV, e));
            }
        };

        if let Err(e) = set_link_up(handle, LOOPBACK_DEV).await {
            return Err(anyhow!(
                "Failed to set link up for device {}. Error={}",
                LOOPBACK_DEV,
                e
            ));
        }

        Ok(())
    }

    // TODO: design network config struct
    async fn configure_nic(
        &self,
        handle: &rtnetlink::Handle,
    ) -> anyhow::Result<()> {
        if let Ok(ipv6) =
            format!("{}{}", DEFAULT_NET_DEV_IPV6, DEFAULT_NET_DEV_IPV6_SUBNET)
                .parse::<Ipv6Network>()
        {
            if let Err(e) = add_address(DEFAULT_NET_DEV, ipv6, handle).await {
                return Err(anyhow!(
                    "Failed to add ipv6 address to device {}. Error={}",
                    DEFAULT_NET_DEV,
                    e
                ));
            }

            if let Err(e) = set_link_up(handle, DEFAULT_NET_DEV).await {
                return Err(anyhow!(
                    "Failed to set link up for device {}. Error={}",
                    DEFAULT_NET_DEV,
                    e
                ));
            }

            if let Ok(destv6) = "::/0".to_string().parse::<Ipv6Network>() {
                if let Err(e) =
                    add_route_v6(&destv6, DEFAULT_NET_DEV, &ipv6, handle).await
                {
                    return Err(anyhow!(
                        "Failed to add ipv6 route to device {}. Error={}",
                        DEFAULT_NET_DEV,
                        e
                    ));
                }
            }
        };

        Ok(())
    }

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
                    "Failed to spawn power button device listener. Error={}",
                    e
                );
            }
        }

        // ---- MAIN DAEMON THREAD POOL ----
    }
}

#[async_trait]
impl SystemRuntime for Pid1SystemRuntime {
    async fn init(self, logger_level: Level) -> Result<(), InitError> {
        println!("{}", BANNER);

        logging::init(logger_level)?;
        trace!("Logging started");

        trace!("Configure filesystem");
        fs::mount_vfs("none", "/dev", "devtmpfs")?;
        fs::mount_vfs("none", "/sys", "sysfs")?;
        fs::mount_vfs("proc", "/proc", "proc")?;

        trace!("configure network");
        //show_dir("/sys/class/net/", false); // Show available network interfaces
        match new_connection() {
            Ok((connection, handle, ..)) => {
                self.init_network(connection, &handle).await;
            }
            Err(e) => {
                error!("Could not initialize network! Error={}", e);
            }
        };

        self.spawn_system_runtime_threads();

        trace!("init of auraed as pid1 done");
        Ok(())
    }
}

pub(crate) struct PidGt1SystemRuntime;

#[async_trait]
impl SystemRuntime for PidGt1SystemRuntime {
    async fn init(self, logger_level: Level) -> Result<(), InitError> {
        logging::init(logger_level)?;
        Ok(())
    }
}
