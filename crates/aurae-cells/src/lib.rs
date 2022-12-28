use aurae_executables::auraed::SharedNamespaces;
use cell::Cell;
pub use cell_name::CellName;
pub use cells::Cells;
pub use cgroups::{CgroupSpec, CpuCpus, CpuQuota, CpuWeight, CpusetMems};
pub use error::{CellsError, Result};

mod cell;
mod cell_name;
mod cells;
mod cgroups;
mod error;

#[derive(Debug, Clone)]
pub struct CellSpec {
    pub cgroup_spec: CgroupSpec,
    pub shared_namespaces: SharedNamespaces,
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
            shared_namespaces: Default::default(), // nothing shared in default
        }
    }
}
