/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
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
