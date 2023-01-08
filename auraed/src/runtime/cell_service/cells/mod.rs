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

use crate::runtime::cell_service::executables::auraed::IsolationControls;
use cell::Cell;
pub use cell_name::CellName;
pub use cells::Cells;
use cgroups::Cgroup;
pub use cgroups::{CgroupSpec, CpuCpus, CpuQuota, CpuWeight, CpusetMems};
pub use error::{CellsError, Result};

mod cell;
mod cell_name;
#[allow(clippy::module_inception)]
mod cells;
mod cgroups;
mod error;

#[derive(Debug, Clone)]
pub struct CellSpec {
    pub cgroup_spec: CgroupSpec,
    pub iso_ctl: IsolationControls,
}

impl CellSpec {
    #[cfg(test)]
    pub(crate) fn new_for_tests() -> Self {
        Self {
            cgroup_spec: CgroupSpec {
                cpu_cpus: CpuCpus::new("".into()),
                cpu_quota: CpuQuota::new(0),
                cpu_weight: CpuWeight::new(0),
                cpuset_mems: CpusetMems::new("".into()),
            },
            iso_ctl: IsolationControls {
                isolate_network: true,
                isolate_process: true,
            },
        }
    }
}
