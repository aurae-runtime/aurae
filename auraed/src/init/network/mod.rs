/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
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
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use anyhow::anyhow;
use futures::stream::TryStreamExt;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use log::{error, info, trace, warn};
use netlink_packet_route::LinkMessage;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str;
use std::thread;
use std::time::Duration;

use netlink_packet_route::rtnl::link::nlas::Nla;
use rtnetlink::Handle;

mod sriov;

pub(crate) async fn set_link_up(
    handle: &Handle,
    iface: &str,
) -> anyhow::Result<()> {
    let mut links = handle.link().get().match_name(iface.to_string()).execute();

    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).up().execute().await?
    } else {
        return Err(anyhow!("iface '{}' not found", iface));
    }

    // TODO: replace sleep with an await mechanism that checks if device is up (with a timeout)
    // TODO: https://github.com/aurae-runtime/auraed/issues/40
    info!("Waiting for link '{}' to become up", iface);
    thread::sleep(Duration::from_secs(3));
    info!("Waited 3 seconds, assuming link '{}' is up", iface);

    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn set_link_down(
    handle: &Handle,
    iface: &str,
) -> anyhow::Result<()> {
    let mut links = handle.link().get().match_name(iface.to_string()).execute();

    if let Some(link) = links.try_next().await? {
        handle.link().set(link.header.index).down().execute().await?
    } else {
        return Err(anyhow!("iface '{}' not found", iface));
    }
    trace!("Set link {} down", iface);
    Ok(())
}

pub(crate) async fn add_address(
    iface: &str,
    ip: impl Into<IpNetwork>,
    handle: &Handle,
) -> anyhow::Result<()> {
    let ip = ip.into();

    let mut links = handle.link().get().match_name(iface.to_string()).execute();

    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }
    trace!("Added address to link {}", iface);
    Ok(())
}

pub(crate) async fn get_links(
    handle: &Handle,
) -> anyhow::Result<HashMap<u32, String>> {
    let mut result = HashMap::new();
    let mut links = handle.link().get().execute();

    'outer: while let Some(msg) = links.try_next().await? {
        for nla in msg.nlas.into_iter() {
            if let Nla::IfName(name) = nla {
                result.insert(msg.header.index, name);
                continue 'outer;
            }
        }
        warn!("link with index {} has no name", msg.header.index);
    }

    Ok(result)
}

async fn get_link_msg(
    iface: impl Into<String>,
    handle: &Handle,
) -> anyhow::Result<LinkMessage> {
    match handle
        .link()
        .get()
        .match_name(iface.into())
        .execute()
        .try_next()
        .await
    {
        Ok(link_msg) => match link_msg {
            Some(val) => Ok(val),
            None => {
                Err(anyhow!("Could not retreive link message. Does not exist"))
            }
        },
        Err(e) => Err(anyhow!("Could not retreive link message. Error={}", e)),
    }
}

async fn get_iface_idx(iface: &str, handle: &Handle) -> anyhow::Result<u32> {
    match get_link_msg(iface, handle).await {
        Ok(link_msg) => Ok(link_msg.header.index),
        Err(e) => Err(e),
    }
}

pub(crate) async fn add_route_v6(
    dest: &Ipv6Network,
    iface: &str,
    source: &Ipv6Network,
    handle: &Handle,
) -> anyhow::Result<()> {
    match get_iface_idx(iface, handle).await {
        Ok(iface_idx) => {
            handle
                .route()
                .add()
                .v6()
                .destination_prefix(dest.ip(), dest.prefix())
                .output_interface(iface_idx)
                .pref_source(source.ip())
                .execute()
                .await?;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn add_route_v4(
    dest: &Ipv4Network,
    iface: &str,
    source: &Ipv4Network,
    handle: &Handle,
) -> anyhow::Result<()> {
    match get_iface_idx(iface, handle).await {
        Ok(iface_idx) => {
            handle
                .route()
                .add()
                .v4()
                .destination_prefix(dest.ip(), dest.prefix())
                .output_interface(iface_idx)
                .pref_source(source.ip())
                .execute()
                .await?;
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

pub(crate) async fn dump_addresses(
    handle: &Handle,
    iface: &str,
) -> anyhow::Result<()> {
    let mut links = handle.link().get().match_name(iface.to_string()).execute();
    if let Some(link_msg) = links.try_next().await? {
        info!("{}:", iface);
        for nla in link_msg.nlas.into_iter() {
            if let Nla::IfName(name) = nla {
                info!("\tindex: {}", link_msg.header.index);
                info!("\tname: {}", name);
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
                                    warn!("Could not Convert vec: {:?} to ipv4 or ipv6", addr);
                                    None
                                })
                        });

                    match &ip_addr {
                        Some(IpAddr::V4(ip_addr)) => {
                            info!("\t ipv4: {}", ip_addr);
                        }
                        Some(IpAddr::V6(ip_addr)) => {
                            info!("\t ipv6: {}", ip_addr);
                        }
                        None => {}
                    }
                }
            }
        }
        Ok(())
    } else {
        Err(anyhow!("link {} not found", iface))
    }
}

pub(crate) async fn show_network_info(handle: &Handle) {
    info!("=== Network Interfaces ===");

    info!("Addresses:");
    let links_result = get_links(handle).await;

    match links_result {
        Ok(links) => {
            for (_, iface) in links {
                if let Err(e) = dump_addresses(handle, &iface).await {
                    error!(
                        "Could not dump addresses for iface {}. Error={}",
                        iface, e
                    );
                };
            }
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
    info!("==========================");
}
