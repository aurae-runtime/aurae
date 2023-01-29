use super::cgroups::cpu::CpuController;
use super::cgroups::cpuset::CpusetController;
use super::CellName;

#[derive(Clone)]
pub struct CellInfo {
    pub cell_name: CellName,
    pub cpu: Option<CpuController>,
    pub cpuset: Option<CpusetController>,
    pub isolate_network: bool,
    pub isolate_process: bool,
}
