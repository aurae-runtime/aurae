/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
use nix::unistd::Pid;
use std::path::PathBuf;

/// This is used as the denominator for the CPU quota/period configuration.  This allows users to
/// set the quota as if it was in the unit "µs/s" without worrying about also setting the period.
const MICROSECONDS_PER_SECOND: u64 = 1000000;

#[derive(Debug)]
pub struct Cgroup {
    cell_name: CellName,
    non_leaf: cgroups_rs::Cgroup,
    leaf: cgroups_rs::Cgroup,
}

impl Cgroup {
    pub fn new(cell_name: CellName, spec: CgroupSpec) -> Self {
        let CgroupSpec { cpu, cpuset } = spec;

        // Note: Cgroups v2 "no internal processes" rule.
        // Docs: https://man7.org/linux/man-pages/man7/cgroups.7.html
        // TLDR: "...with the exception of the root cgroup, processes may reside only
        //        in leaf nodes (cgroups that do not themselves contain child cgroups)."

        // First we create the non-leaf cgroup using the spec
        let mut name = cell_name.to_string();
        let builder = CgroupBuilder::new(&name);

        // cpu controller
        let builder = if let Some(CpuController { weight, max }) = cpu {
            let builder = builder.cpu();

            let builder = if let Some(weight) = weight {
                builder.shares(weight.into_inner())
            } else {
                builder
            };

            let builder = if let Some(max) = max {
                builder.quota(max.into_inner()).period(MICROSECONDS_PER_SECOND) // microseconds in a second
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

        let non_leaf = builder.build(hierarchy()).expect("valid cgroup");

        // Now, create the leaf cgroup where we can run processes.
        // NOTE: '_' is a disallowed character in CellName, so won't collide
        name.push_str("/_");
        let leaf =
            CgroupBuilder::new(&name).build(hierarchy()).expect("valid cgroup");

        Self { cell_name, non_leaf, leaf }
    }

    pub fn add_task(&self, pid: Pid) -> cgroups_rs::error::Result<()> {
        self.leaf.add_task_by_tgid((pid.as_raw() as u64).into())
    }

    pub fn delete(&self) -> cgroups_rs::error::Result<()> {
        self.leaf.delete()?;
        self.non_leaf.delete()
    }

    pub fn v2(&self) -> bool {
        self.non_leaf.v2()
    }

    pub fn exists(cell_name: &CellName) -> bool {
        let mut path = PathBuf::from("/sys/fs/cgroup");
        path.push(cell_name.as_inner());
        path.exists()
    }
}

fn hierarchy() -> Box<dyn Hierarchy> {
    // Auraed will assume the V2 cgroup hierarchy by default.
    // For now, we do not change this, albeit in theory we could
    // likely create backwards compatability for V1 hierarchy.
    //
    // For now, we simply... don't.
    // hierarchies::auto() // Uncomment to auto detect Cgroup hierarchy
    // hierarchies::V2
    Box::new(hierarchies::V2::new())
}
