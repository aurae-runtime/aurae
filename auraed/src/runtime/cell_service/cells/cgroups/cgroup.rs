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

use crate::runtime::cell_service::cells::{
    cgroups::{CpuController, CpusetController},
    CellName, CgroupSpec,
};
use cgroups_rs::{cgroup_builder::CgroupBuilder, hierarchies, Hierarchy};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

/// This is used as the denominator for the CPU quota/period configuration.  This allows users to
/// set the quota as if it was in the unit "µs/s" without worrying about also setting the period.
const MICROSECONDS_PER_SECOND: u64 = 1000000;

#[derive(Debug)]
pub struct Cgroup {
    cell_name: CellName,
    inner: cgroups_rs::Cgroup,
}

impl Cgroup {
    pub fn new(cell_name: CellName, spec: CgroupSpec) -> Self {
        let CgroupSpec { cpu, cpuset } = spec;

        // NOTE: v2 cgroups can either have nested cgroups or processes, not both (leaf workaround)
        // NOTE: '_' is a disallowed character in cell name, so won't collide
        let name = format!("{cell_name}/_");

        let builder = CgroupBuilder::new(&name);

        // cpu controller
        let builder = if let Some(CpuController { weight, limit }) = cpu {
            let builder = builder.cpu();

            let builder = if let Some(weight) = weight {
                builder.shares(weight.into_inner())
            } else {
                builder
            };

            let builder = if let Some(limit) = limit {
                builder.quota(limit.into_inner()).period(MICROSECONDS_PER_SECOND) // microseconds in a second
            } else {
                builder
            };

            builder.done()
        } else {
            builder
        };

        // cpuset controller
        let builder = if let Some(CpusetController { cpus, mems }) = cpuset {
            let builder = builder.cpu();

            let builder = if let Some(cpus) = cpus {
                builder.cpus(cpus.into_inner())
            } else {
                builder
            };

            let builder = if let Some(mems) = mems {
                builder.mems(mems.into_inner())
            } else {
                builder
            };

            builder.done()
        } else {
            builder
        };

        let inner = builder.build(hierarchy()).expect("valid cgroup");

        Self { cell_name, inner }
    }

    pub fn delete(&self) -> cgroups_rs::error::Result<()> {
        self.inner.delete()?;

        // The cgroup was made as {CellName}/_ to work around the limitations of v2 cgroups.
        //       But when we are deleting the cgroup, we are leaving behind a cgroup
        //       at {CellName}. We need to clean that up.
        cgroups_rs::Cgroup::load(hierarchy(), &*self.cell_name).delete()
    }

    pub fn exists(cell_name: &CellName) -> bool {
        let mut path = PathBuf::from("/sys/fs/cgroup");
        path.push(cell_name.deref());
        path.exists()
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
