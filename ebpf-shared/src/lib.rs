#![no_std]

pub trait HasCgroup {
    fn cgroup_id(&self) -> u64;
}

pub trait HasHostPid {
    fn pid(&self) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal {
    pub cgroup_id: u64,
    pub signum: i32,
    pub pid: i32,
}

impl HasCgroup for Signal {
    fn cgroup_id(&self) -> u64 {
        self.cgroup_id
    }
}

impl HasHostPid for Signal {
    fn pid(&self) -> i32 {
        self.pid
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForkedProcess {
    pub parent_pid: i32,
    pub child_pid: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessExit {
    pub pid: i32,
}
