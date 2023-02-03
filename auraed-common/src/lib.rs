#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Signal {
    pub signr: i32,
    pub pid: u32,
}
