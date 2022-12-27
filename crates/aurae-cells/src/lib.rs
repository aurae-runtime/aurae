use aurae_executables::SharedNamespaces;
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
