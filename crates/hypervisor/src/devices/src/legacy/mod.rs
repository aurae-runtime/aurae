// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
#[cfg(target_arch = "x86_64")]
mod i8042;
#[cfg(target_arch = "aarch64")]
mod rtc;
mod serial;
#[cfg(target_arch = "x86_64")]
pub use i8042::I8042Wrapper;
#[cfg(target_arch = "aarch64")]
pub use rtc::RtcWrapper;
pub use serial::Error as SerialError;
pub use serial::SerialWrapper;
use std::io;
use std::ops::Deref;

use vm_superio::Trigger;
use vmm_sys_util::eventfd::EventFd;

/// Newtype for implementing the trigger functionality for `EventFd`.
///
/// The trigger is used for handling events in the legacy devices.
pub struct EventFdTrigger(EventFd);

impl Trigger for EventFdTrigger {
    type E = io::Error;

    fn trigger(&self) -> io::Result<()> {
        self.write(1)
    }
}
impl Deref for EventFdTrigger {
    type Target = EventFd;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl EventFdTrigger {
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(EventFdTrigger((**self).try_clone()?))
    }
    pub fn new(flag: i32) -> io::Result<Self> {
        let event_fd = EventFd::new(flag)?;
        Ok(EventFdTrigger(event_fd))
    }
}
