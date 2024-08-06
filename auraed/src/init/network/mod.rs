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

use futures::stream::TryStreamExt;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use netlink_packet_route::rtnl::link::nlas::Nla;
use rtnetlink::Handle;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str;
use std::thread;
use std::time::Duration;
use tracing::{error, info, trace, warn};

mod sriov;

#[derive(thiserror::Error, Debug)]
pub(crate) enum NetworkError {
    #[error("Failed to initialize network: {0}")]
    FailedToConnect(#[from] std::io::Error),
    #[error("Could not find link `{iface}`")]
    DeviceNotFound { iface: String },
    #[error("Error adding address `{ip}` to link `{iface}`: {source}")]
    ErrorAddingAddress {
        iface: String,
        ip: IpNetwork,
        source: rtnetlink::Error,
    },
    #[error("Failed to set link up for device `{iface}`: {source}")]
    ErrorSettingLinkUp { iface: String, source: rtnetlink::Error },
    #[error("Failed to set link down for device `{iface}`: {source}")]
    ErrorSettingLinkDown { iface: String, source: rtnetlink::Error },
    #[error("Error adding route from `{route_source}` to {route_destination}` for device `{iface}`: {source}")]
    ErrorAddingRoute {
        iface: String,
        route_source: IpNetwork,
        route_destination: IpNetwork,
        source: rtnetlink::Error,
    },
    #[error(transparent)]
    Other(#[from] rtnetlink::Error),
}

pub(crate) struct Network(Handle);

impl Network {
    pub(crate) fn connect() -> Result<Network, NetworkError> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        let _ignored = tokio::spawn(connection);
        Ok(Self(handle))
    }

    pub(crate) async fn init(&self) -> Result<(), NetworkError> {
        configure_loopback(&self.0).await?;
        configure_nic(&self.0).await?;
        Ok(())
    }

    pub(crate) async fn show_network_info(&self) {
        info!("=== Network Interfaces ===");

        info!("Addresses:");
        let links_result = get_links(&self.0).await;

        match links_result {
            Ok(links) => {
                for (_, iface) in links {
                    if let Err(e) = dump_addresses(&self.0, &iface).await {
                        error!(
                            "Could not dump addresses for iface {iface}. Error={e:?}"
                        );
                    };
                }
            }
            Err(e) => {
                error!("{e:?}");
            }
        }
        info!("==========================");
    }
}

async fn configure_loopback(handle: &Handle) -> Result<(), NetworkError> {
    const LOOPBACK_DEV: &str = "lo";
    const LOOPBACK_IPV6: &str = "::1";
    const LOOPBACK_IPV6_SUBNET: &str = "/128";
    const LOOPBACK_IPV4: &str = "127.0.0.1";
    const LOOPBACK_IPV4_SUBNET: &str = "/8";

    trace!("configure {LOOPBACK_DEV}");

    add_address(
        handle,
        LOOPBACK_DEV.to_owned(),
        format!("{LOOPBACK_IPV6}{LOOPBACK_IPV6_SUBNET}")
            .parse::<Ipv6Network>()
            .expect("valid ipv6 address"),
    )
    .await?;

    add_address(
        handle,
        LOOPBACK_DEV.to_owned(),
        format!("{LOOPBACK_IPV4}{LOOPBACK_IPV4_SUBNET}")
            .parse::<Ipv4Network>()
            .expect("valid ipv4 address"),
    )
    .await?;

    set_link_up(handle, LOOPBACK_DEV.to_owned()).await?;

    info!("Successfully configured {}", LOOPBACK_DEV);
    Ok(())
}

// TODO: design network config struct
async fn configure_nic(handle: &Handle) -> Result<(), NetworkError> {
    const DEFAULT_NET_DEV: &str = "eth0";
    const DEFAULT_NET_DEV_IPV6: &str = "fe80::2";
    const DEFAULT_NET_DEV_IPV6_GATEWAY: &str = "fe80::1";
    const DEFAULT_NET_DEV_IPV6_SUBNET: &str = "/64";

    trace!("configure {DEFAULT_NET_DEV}");

    let ipv6_addr =
        format!("{DEFAULT_NET_DEV_IPV6}{DEFAULT_NET_DEV_IPV6_SUBNET}")
            .parse::<Ipv6Network>()
            .expect("valid ipv6 address");

    let gateway = DEFAULT_NET_DEV_IPV6_GATEWAY
        .to_string()
        .parse::<Ipv6Network>()
        .expect("gateway");

    add_address(handle, DEFAULT_NET_DEV.to_owned(), ipv6_addr).await?;

    set_link_up(handle, DEFAULT_NET_DEV.to_owned()).await?;

    add_route_v6(
        handle,
        DEFAULT_NET_DEV.to_owned(),
        "::/0".parse::<Ipv6Network>().expect("valid ipv6 address"),
        gateway,
    )
    .await?;

    info!("Successfully configured {DEFAULT_NET_DEV}");
    Ok(())
}

async fn add_address(
    handle: &Handle,
    iface: String,
    ip: impl Into<IpNetwork>,
) -> Result<(), NetworkError> {
    let ip = ip.into();
    let link_index = get_link_index(handle, iface.clone()).await?;

    handle
        .address()
        .add(link_index, ip.ip(), ip.prefix())
        .execute()
        .await
        .map(|_| {
            trace!("Added address to link {iface}");
        })
        .map_err(|e| NetworkError::ErrorAddingAddress {
            iface,
            ip,
            source: e,
        })?;

    Ok(())
}

async fn set_link_up(
    handle: &Handle,
    iface: String,
) -> Result<(), NetworkError> {
    let link_index = get_link_index(handle, iface.clone()).await?;

    handle
        .link()
        .set(link_index)
        .up()
        .execute()
        .await
        .map(|_| {
            // TODO: replace sleep with an await mechanism that checks if device is up (with a timeout)
            // TODO: https://github.com/aurae-runtime/auraed/issues/40
            info!("Waiting for link '{iface}' to become up");
            thread::sleep(Duration::from_secs(3));
            info!("Waited 3 seconds, assuming link '{iface}' is up");
        })
        .map_err(|e| NetworkError::ErrorSettingLinkUp { iface, source: e })
}

#[allow(unused)]
async fn set_link_down(
    handle: &Handle,
    iface: String,
) -> Result<(), NetworkError> {
    let link_index = get_link_index(handle, iface.clone()).await?;

    handle
        .link()
        .set(link_index)
        .down()
        .execute()
        .await
        .map(|_| {
            trace!("Set link {iface} down");
        })
        .map_err(|e| NetworkError::ErrorSettingLinkDown { iface, source: e })
}

async fn get_link_index(
    handle: &Handle,
    iface: String,
) -> Result<u32, NetworkError> {
    let link = handle
        .link()
        .get()
        .match_name(iface.clone())
        .execute()
        .try_next()
        .await;

    if let Ok(Some(link)) = link {
        Ok(link.header.index)
    } else {
        Err(NetworkError::DeviceNotFound { iface })
    }
}

#[allow(unused)]
async fn add_route_v4(
    handle: &Handle,
    iface: String,
    source: Ipv4Network,
    dest: Ipv4Network,
) -> Result<(), NetworkError> {
    let link_index = get_link_index(handle, iface.clone()).await?;

    handle
        .route()
        .add()
        .v4()
        .destination_prefix(dest.ip(), dest.prefix())
        .output_interface(link_index)
        .pref_source(source.ip())
        .execute()
        .await
        .map_err(|e| NetworkError::ErrorAddingRoute {
            iface,
            route_source: source.into(),
            route_destination: dest.into(),
            source: e,
        })?;

    Ok(())
}

async fn add_route_v6(
    handle: &Handle,
    iface: String,
    source: Ipv6Network,
    dest: Ipv6Network,
) -> Result<(), NetworkError> {
    let link_index = get_link_index(handle, iface.clone()).await?;

    handle
        .route()
        .add()
        .v6()
        .source_prefix(source.ip(), source.prefix())
        .gateway(dest.ip())
        .output_interface(link_index)
        .execute()
        .await
        .map_err(|e| NetworkError::ErrorAddingRoute {
            iface,
            route_source: source.into(),
            route_destination: dest.into(),
            source: e,
        })?;

    Ok(())
}

async fn get_links(
    handle: &Handle,
) -> Result<HashMap<u32, String>, NetworkError> {
    let mut result = HashMap::new();
    let mut links = handle.link().get().execute();

    'outer: while let Some(link_msg) = links.try_next().await? {
        for nla in link_msg.nlas.into_iter() {
            if let Nla::IfName(name) = nla {
                let _ = result.insert(link_msg.header.index, name);
                continue 'outer;
            }
        }
        warn!("link with index {} has no name", link_msg.header.index);
    }

    Ok(result)
}

async fn dump_addresses(
    handle: &Handle,
    iface: &str,
) -> Result<(), NetworkError> {
    let mut links = handle.link().get().match_name(iface.to_string()).execute();
    if let Some(link_msg) = links.try_next().await? {
        info!("{}:", iface);
        for nla in link_msg.nlas.into_iter() {
            if let Nla::IfName(name) = nla {
                info!("\tindex: {}", link_msg.header.index);
                info!("\tname: {name}");
            }
        }

        let mut address_msg = handle
            .address()
            .get()
            .set_link_index_filter(link_msg.header.index)
            .execute();

        while let Some(msg) = address_msg.try_next().await? {
            for nla_address in msg.nlas.into_iter() {
                if let netlink_packet_route::address::Nla::Address(addr) =
                    nla_address
                {
                    let ip_addr = addr.try_into()
                        .map(|ip: [u8; 4]| Some(IpAddr::from(ip)))
                        .unwrap_or_else(|addr| {
                            addr.try_into()
                                .map(|ip: [u8; 16]| Some(IpAddr::from(ip)))
                                .unwrap_or_else(|addr| {
                                    warn!("Could not Convert vec: {addr:?} to ipv4 or ipv6");
                                    None
                                })
                        });

                    match &ip_addr {
                        Some(IpAddr::V4(ip_addr)) => {
                            info!("\t ipv4: {ip_addr}");
                        }
                        Some(IpAddr::V6(ip_addr)) => {
                            info!("\t ipv6: {ip_addr}");
                        }
                        None => {}
                    }
                }
            }
        }
        Ok(())
    } else {
        Err(NetworkError::DeviceNotFound { iface: iface.to_string() })
    }
}