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

use crate::{CellName, CgroupSpec};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::{hierarchies, Hierarchy};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Cgroup {
    cell_name: CellName,
    inner: cgroups_rs::Cgroup,
}

impl Cgroup {
    pub fn new(cell_name: CellName, spec: CgroupSpec) -> Self {
        let CgroupSpec { cpu_cpus, cpu_quota, cpu_weight, cpuset_mems } = spec;

        // NOTE: v2 cgroups can either have nested cgroups or processes, not both (leaf workaround)
        // NOTE: '_' is a disallowed character in cell name, so won't collide
        let name = format!("{cell_name}/_");
        let hierarchy = hierarchy();

        let inner = CgroupBuilder::new(&name)
            // CPU Controller
            .cpu()
            .shares(cpu_weight.into_inner())
            .mems(cpuset_mems.into_inner())
            .period(1000000) // microseconds in a second
            .quota(cpu_quota.into_inner())
            .cpus(cpu_cpus.into_inner())
            .done()
            // Final Build
            .build(hierarchy);

        Self { cell_name, inner }
    }

    pub fn delete(&self) -> cgroups_rs::error::Result<()> {
        self.inner.delete()?;

        // The cgroup was made as {CellName}/_ to work around the limitations of v2 cgroups.
        //       But when we are deleting the cgroup, we are leaving behind a cgroup
        //       at {CellName}. We need to clean that up.
        cgroups_rs::Cgroup::load(hierarchy(), &*self.cell_name).delete()
    }
}

impl Deref for Cgroup {
    type Target = cgroups_rs::Cgroup;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Cgroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn hierarchy() -> Box<dyn Hierarchy> {
    // Auraed will assume the V2 cgroup hierarchy by default.
    // For now we do not change this, albeit in theory we could
    // likely create backwards compatability for V1 hierarchy.
    //
    // For now, we simply... don't.
    // hierarchies::auto() // Uncomment to auto detect Cgroup hierarchy
    // hierarchies::V2
    Box::new(hierarchies::V2::new())
}
