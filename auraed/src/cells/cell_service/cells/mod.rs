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

pub use cell::Cell;
pub use cell_name::CellName;
pub use cells::Cells;
pub use cells_cache::CellsCache;
use cgroups::CgroupSpec;
pub use error::{CellsError, Result};
pub use nested_auraed::IsolationControls;

mod cell;
mod cell_name;
#[allow(clippy::module_inception)]
mod cells;
mod cells_cache;
pub mod cgroups;
mod error;
mod nested_auraed;

#[derive(Debug, Clone)]
pub struct CellSpec {
    pub cgroup_spec: CgroupSpec,
    pub iso_ctl: IsolationControls,
}

impl CellSpec {
    #[cfg(test)]
    pub(crate) fn new_for_tests() -> Self {
        use crate::cells::cell_service::cells::cgroups::{
            CpuController, Limit, MemoryController, Weight,
        };

        Self {
            cgroup_spec: CgroupSpec {
                cpu: Some(CpuController {
                    weight: Some(Weight::new(100)),
                    max: None,
                    period: Some(100000),
                }),
                cpuset: None,
                memory: Some(MemoryController {
                    min: None,
                    low: None,
                    high: None,
                    max: Some(Limit::new(1000000)),
                }),
            },
            iso_ctl: IsolationControls {
                isolate_network: false,
                isolate_process: false,
            },
        }
    }
}