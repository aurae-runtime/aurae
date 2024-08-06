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
use std::num::ParseIntError;
use std::{cmp, fs, io};

#[derive(thiserror::Error, Debug)]
pub(crate) enum SriovError {
    #[error("Failed to get sriov capabilities of device {iface}")]
    GetCapabilitiesFailure { iface: String, source: io::Error },
    #[error("Failed to parse sriov capabilities {capabilities}")]
    ParseCapabilitiesFailure { capabilities: String, source: ParseIntError },
}

// Create max(limit, max possible sriov for given iface) sriov devices for the given iface
#[allow(dead_code)]
pub(crate) fn setup_sriov(iface: &str, limit: u16) -> Result<(), SriovError> {
    if limit == 0 {
        return Ok(());
    }

    let sriov_totalvfs = get_sriov_capabilities(iface).map_err(|e| {
        SriovError::GetCapabilitiesFailure {
            iface: iface.to_owned(),
            source: e,
        }
    })?;

    let sriov_totalvfs =
        sriov_totalvfs.trim_end().parse::<u16>().map_err(|e| {
            SriovError::ParseCapabilitiesFailure {
                capabilities: sriov_totalvfs,
                source: e,
            }
        })?;

    let num = cmp::min(limit, sriov_totalvfs);

    fs::write(
        format!("/sys/class/net/{iface}/device/sriov_numvfs"),
        num.to_string(),
    )
    .expect("Unable to write file");
    Ok(())
}

fn get_sriov_capabilities(iface: &str) -> Result<String, io::Error> {
    fs::read_to_string(format!("/sys/class/net/{iface}/device/sriov_totalvfs"))
}