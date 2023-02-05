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

use crate::cells::cell_service::cells::{
    cgroups::{CpuController, CpusetController, MemoryController},
    CellName, CgroupSpec,
};
use libcgroups::common::{CgroupManager, ControllerOpt, DEFAULT_CGROUP_ROOT};
use libcgroups::stats::Stats;
use libcgroups::v2;
use nix::unistd::Pid;
use oci_spec::runtime::{
    LinuxCpuBuilder, LinuxMemoryBuilder, LinuxResourcesBuilder,
};
use std::path::PathBuf;
use std::str::FromStr;

use super::error::{CgroupsError, Result};

/// This is used as the denominator for the CPU quota/period configuration.  This allows users to
/// set the quota as if it was in the unit "µs/s" without worrying about also setting the period.
const MICROSECONDS_PER_SECOND: u64 = 1000000;

#[derive(Debug)]
pub struct Cgroup {
    cell_name: CellName,
}

impl Cgroup {
    pub fn new(
        cell_name: CellName,
        spec: CgroupSpec,
        nested_auraed_pid: Pid,
    ) -> Result<Self> {
        let CgroupSpec { cpu, cpuset, memory } = spec;

        // Note: Cgroups v2 "no internal processes" rule.
        // Docs: https://man7.org/linux/man-pages/man7/cgroups.7.html
        // TLDR: "...with the exception of the root cgroup, processes may reside only
        //        in leaf nodes (cgroups that do not themselves contain child cgroups)."

        // First we create the cgroup managers. This doesn't do anything on the system.
        let non_leaf = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            cell_name.clone().into_inner(),
        )
        .expect("valid cgroup");

        let leaf = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            get_leaf_path(&cell_name),
        )
        .expect("valid cgroup");

        // libcgroups will only create the cgroup when the first task is added,
        // so we need to add a task before applying the controllers.
        if let Err(e) = leaf.add_task(nested_auraed_pid) {
            let _ = leaf.remove();
            let _ = non_leaf.remove();
            return Err(CgroupsError::AddTaskToCgroup { cell_name, source: e });
        }

        let builder = LinuxResourcesBuilder::default();

        // oci_spec, which libcgroups uses, combines the cpu and cpuset controllers
        let builder = if cpu.is_some() || cpuset.is_some() || memory.is_some() {
            let cpu_builder = LinuxCpuBuilder::default();

            // cpu controller
            let cpu_builder = if let Some(CpuController { weight, max }) = cpu {
                let cpu_builder = if let Some(weight) = weight {
                    cpu_builder.shares(weight.into_inner())
                } else {
                    cpu_builder
                };

                if let Some(max) = max {
                    cpu_builder
                        .quota(max.into_inner())
                        .period(MICROSECONDS_PER_SECOND) // microseconds in a second
                } else {
                    cpu_builder
                }
            } else {
                cpu_builder
            };

            // cpuset controller
            let cpu_builder =
                if let Some(CpusetController { cpus, mems }) = cpuset {
                    let cpu_builder = if let Some(cpus) = cpus {
                        cpu_builder.cpus(cpus.into_inner())
                    } else {
                        cpu_builder
                    };

                    if let Some(mems) = mems {
                        cpu_builder.mems(mems.into_inner())
                    } else {
                        cpu_builder
                    }
                } else {
                    cpu_builder
                };

            let memory_builder = LinuxMemoryBuilder::default();
            let memory_builder =
                if let Some(MemoryController { min: _, low, high: _, max }) =
                    memory
                {
                    let memory_builder = if let Some(low) = low {
                        memory_builder.reservation(low.into_inner())
                    } else {
                        memory_builder
                    };

                    if let Some(max) = max {
                        memory_builder.limit(max.into_inner())
                    } else {
                        memory_builder
                    }
                } else {
                    memory_builder
                };

            let cpu = cpu_builder.build().expect("valid cpu builder");
            let memory = memory_builder.build().expect("valid memory builder");
            builder.cpu(cpu).memory(memory)
        } else {
            builder
        };

        let options = builder.build().expect("valid options");
        let options = ControllerOpt {
            resources: &options,
            disable_oom_killer: false,
            oom_score_adj: None,
            freezer_state: None,
        };

        if let Err(e) = non_leaf.apply(&options) {
            // try to remove, but ignore the error as the original error is more appropriate to return
            // libcgroups takes care of killing any processes it finds
            let _ = leaf.remove();
            let _ = non_leaf.remove();
            return Err(CgroupsError::CreateCgroup { cell_name, source: e });
        }

        Ok(Self { cell_name })
    }

    pub fn add_task(&self, pid: Pid) -> Result<()> {
        let manager = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            get_leaf_path(&self.cell_name),
        )
        .expect("valid cgroup");

        manager.add_task(pid).map_err(|e| CgroupsError::AddTaskToCgroup {
            cell_name: self.cell_name.clone(),
            source: e,
        })
    }

    pub fn delete(&self) -> Result<()> {
        let leaf = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            get_leaf_path(&self.cell_name),
        )
        .expect("valid cgroup");

        leaf.remove().map_err(|e| CgroupsError::DeleteCgroup {
            cell_name: self.cell_name.clone(),
            source: e,
        })?;

        let non_leaf = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            self.cell_name.clone().into_inner(),
        )
        .expect("valid cgroup");

        non_leaf.remove().map_err(|e| CgroupsError::DeleteCgroup {
            cell_name: self.cell_name.clone(),
            source: e,
        })
    }

    pub fn v2(&self) -> bool {
        // Auraed will assume the V2 cgroup hierarchy by default.
        // For now, we do not change this, albeit in theory we could
        // likely create backwards compatability for V1 hierarchy.
        //
        // For now, we simply... don't.
        true
    }

    // TODO: use this
    #[allow(unused)]
    pub fn stats(&self) -> Result<Stats> {
        let non_leaf = v2::manager::Manager::new(
            DEFAULT_CGROUP_ROOT.into(),
            self.cell_name.clone().into_inner(),
        )
        .expect("valid cgroup");

        non_leaf.stats().map_err(|e| CgroupsError::ReadStats {
            cell_name: self.cell_name.clone(),
            source: e,
        })
    }

    pub fn exists(cell_name: &CellName) -> bool {
        let mut path =
            PathBuf::from_str(DEFAULT_CGROUP_ROOT).expect("valid path");
        path.push(cell_name.as_inner());
        path.exists()
    }
}

fn get_leaf_path(cell_name: &CellName) -> PathBuf {
    // '_' is an invalid character in CellName, making it safe to use
    cell_name.as_inner().join("_")
}
