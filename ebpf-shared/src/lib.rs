#![no_std]

pub trait CgroupId {
    fn cgroup_id(&self) -> u64;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal {
    pub cgroup_id: u64,
    pub signum: i32,
    pub pid: i32,
}

impl CgroupId for Signal {
    fn cgroup_id(&self) -> u64 {
        self.cgroup_id
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForkedProcess {
    pub parent_pid: u32,
    pub child_pid: u32,
}
