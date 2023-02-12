#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal {
    pub cgroupid: u64,
    pub signr: i32,
    pub pid: u32,
}
