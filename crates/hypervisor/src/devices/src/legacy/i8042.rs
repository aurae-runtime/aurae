// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::convert::TryInto;
use vm_device::bus::{PioAddress, PioAddressOffset};
use vm_device::MutDevicePio;
use vm_superio::I8042Device;

use utils::debug;

use super::EventFdTrigger;
pub struct I8042Wrapper(pub I8042Device<EventFdTrigger>);

impl MutDevicePio for I8042Wrapper {
    fn pio_read(&mut self, _base: PioAddress, offset: PioAddressOffset, data: &mut [u8]) {
        if data.len() != 1 {
            debug!("Invalid I8042 data length on read: {}", data.len());
            return;
        }
        match offset.try_into() {
            Ok(offset) => {
                self.0.read(offset);
            }
            Err(_) => debug!("Invalid I8042 read offset."),
        }
    }

    fn pio_write(&mut self, _base: PioAddress, offset: PioAddressOffset, data: &[u8]) {
        if data.len() != 1 {
            debug!("Invalid I8042 data length on write: {}", data.len());
            return;
        }
        match offset.try_into() {
            Ok(offset) => {
                if self.0.write(offset, data[0]).is_err() {
                    debug!("Failed to write to I8042.");
                }
            }
            Err(_) => debug!("Invalid I8042 write offset"),
        }
    }
}
