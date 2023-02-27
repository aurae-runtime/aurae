// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::convert::TryInto;
use vm_device::bus::MmioAddress;
use vm_device::MutDeviceMmio;
use vm_superio::{rtc_pl031::NoEvents, Rtc};

use utils::debug;

pub struct RtcWrapper(pub Rtc<NoEvents>);

impl MutDeviceMmio for RtcWrapper {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        if data.len() != 4 {
            debug!("RTC invalid data length on read: {}", data.len());
            return;
        }

        match offset.try_into() {
            // The unwrap() is safe because we checked that `data` has length 4.
            Ok(offset) => self.0.read(offset, data.try_into().unwrap()),
            Err(_) => debug!("Invalid RTC read offset."),
        }
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        if data.len() != 4 {
            debug!("RTC invalid data length on write: {}", data.len());
            return;
        }

        match offset.try_into() {
            // The unwrap() is safe because we checked that `data` has length 4.
            Ok(offset) => self.0.write(offset, data.try_into().unwrap()),
            Err(_) => debug!("Invalid RTC write offset."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_requests() {
        let mut rtc = RtcWrapper(Rtc::new());

        // Check that passing invalid data does not result in a crash.
        let mut invalid_data = [0; 3];
        let valid_offset = 0x0;
        rtc.mmio_read(MmioAddress(0), valid_offset, invalid_data.as_mut());
        rtc.mmio_write(MmioAddress(0), valid_offset, &invalid_data);

        // Check that passing an invalid offset does not result in a crash.
        let valid_data = [0; 4];
        let invalid_offset = u64::MAX;
        rtc.mmio_write(MmioAddress(0), invalid_offset, &valid_data);
    }

    #[test]
    fn test_valid_read() {
        use core::time::Duration;
        use std::thread;

        let mut rtc = RtcWrapper(Rtc::new());
        let mut data = [0; 4];
        let offset = 0x0;

        // Read the data register.
        rtc.mmio_read(MmioAddress(0), offset, data.as_mut());
        let first_read = u32::from_le_bytes(data);

        // Sleep for 1.5 seconds.
        let delay = Duration::from_millis(1500);
        thread::sleep(delay);

        // Read the data register again.
        rtc.mmio_read(MmioAddress(0), offset, data.as_mut());
        let second_read = u32::from_le_bytes(data);

        assert!(second_read > first_read);
    }

    #[test]
    fn test_valid_write() {
        let mut rtc = RtcWrapper(Rtc::new());
        let write_data = [1; 4];
        let mut read_data = [0; 4];
        let offset = 0x8;

        // Write to and read from the load register.
        rtc.mmio_write(MmioAddress(0), offset, &write_data);
        rtc.mmio_read(MmioAddress(0), offset, read_data.as_mut());

        assert_eq!(
            u32::from_le_bytes(write_data),
            u32::from_le_bytes(read_data)
        );
    }
}
