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

pub use cgroup::Cgroup;
pub use cpu::CpuController;
pub use cpuset::CpusetController;
pub use limit::Limit;
pub use memory::MemoryController;
pub use protection::Protection;
pub use weight::Weight;

pub mod cpu;
pub mod cpuset;
pub mod error;
pub mod memory;

mod allocation;
mod cgroup;
mod limit;
mod protection;
mod weight;

#[derive(Debug, Clone)]
pub struct CgroupSpec {
    pub cpu: Option<CpuController>,
    pub cpuset: Option<CpusetController>,
    pub memory: Option<MemoryController>,
}