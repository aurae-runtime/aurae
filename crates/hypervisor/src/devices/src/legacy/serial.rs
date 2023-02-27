// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::convert::TryInto;
use std::io::{self, stdin, Read, Write};

use event_manager::{EventOps, Events, MutEventSubscriber};
#[cfg(target_arch = "aarch64")]
use vm_device::{bus::MmioAddress, MutDeviceMmio};
#[cfg(target_arch = "x86_64")]
use vm_device::{
    bus::{PioAddress, PioAddressOffset},
    MutDevicePio,
};
use vm_superio::serial::{NoEvents, SerialEvents};
use vm_superio::{Serial, Trigger};
use vmm_sys_util::epoll::EventSet;

use utils::debug;

/// Newtype for implementing `event-manager` functionalities.
pub struct SerialWrapper<T: Trigger, EV: SerialEvents, W: Write>(pub Serial<T, EV, W>);

impl<T: Trigger, W: Write> MutEventSubscriber for SerialWrapper<T, NoEvents, W> {
    fn process(&mut self, events: Events, ops: &mut EventOps) {
        // Respond to stdin events.
        // `EventSet::IN` => send what's coming from stdin to the guest.
        // `EventSet::HANG_UP` or `EventSet::ERROR` => deregister the serial input.
        let mut out = [0u8; 32];
        match stdin().read(&mut out) {
            Err(e) => {
                eprintln!("Error while reading stdin: {:?}", e);
            }
            Ok(count) => {
                let event_set = events.event_set();
                let unregister_condition =
                    event_set.contains(EventSet::ERROR) | event_set.contains(EventSet::HANG_UP);
                if count > 0 {
                    if self.0.enqueue_raw_bytes(&out[..count]).is_err() {
                        eprintln!("Failed to send bytes to the guest via serial input");
                    }
                } else if unregister_condition {
                    // Got 0 bytes from serial input; is it a hang-up or error?
                    ops.remove(events)
                        .expect("Failed to unregister serial input");
                }
            }
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        // Hook to stdin events.
        ops.add(Events::new(&stdin(), EventSet::IN))
            .expect("Failed to register serial input event");
    }
}

impl<T: Trigger<E = io::Error>, W: Write> SerialWrapper<T, NoEvents, W> {
    fn bus_read(&mut self, offset: u8, data: &mut [u8]) {
        if data.len() != 1 {
            debug!("Serial console invalid data length on read: {}", data.len());
            return;
        }

        // This is safe because we checked that `data` has length 1.
        data[0] = self.0.read(offset);
    }

    fn bus_write(&mut self, offset: u8, data: &[u8]) {
        if data.len() != 1 {
            debug!(
                "Serial console invalid data length on write: {}",
                data.len()
            );
            return;
        }

        // This is safe because we checked that `data` has length 1.
        let res = self.0.write(offset, data[0]);
        if res.is_err() {
            debug!("Error writing to serial console: {:#?}", res.unwrap_err());
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl<T: Trigger<E = io::Error>, W: Write> MutDevicePio for SerialWrapper<T, NoEvents, W> {
    fn pio_read(&mut self, _base: PioAddress, offset: PioAddressOffset, data: &mut [u8]) {
        // TODO: this function can't return an Err, so we'll mark error conditions
        // (data being more than 1 byte, offset overflowing an u8) with logs & metrics.

        match offset.try_into() {
            Ok(offset) => self.bus_read(offset, data),
            Err(_) => debug!("Invalid serial console read offset."),
        }
    }

    fn pio_write(&mut self, _base: PioAddress, offset: PioAddressOffset, data: &[u8]) {
        // TODO: this function can't return an Err, so we'll mark error conditions
        // (data being more than 1 byte, offset overflowing an u8) with logs & metrics.

        match offset.try_into() {
            Ok(offset) => self.bus_write(offset, data),
            Err(_) => debug!("Invalid serial console write offset."),
        }
    }
}

#[cfg(target_arch = "aarch64")]
impl<T: Trigger<E = io::Error>, W: Write> MutDeviceMmio for SerialWrapper<T, NoEvents, W> {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        // TODO: this function can't return an Err, so we'll mark error conditions
        // (data being more than 1 byte, offset overflowing an u8) with logs & metrics.

        match offset.try_into() {
            Ok(offset) => self.bus_read(offset, data),
            Err(_) => debug!("Invalid serial console read offset."),
        }
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        // TODO: this function can't return an Err, so we'll mark error conditions
        // (data being more than 1 byte, offset overflowing an u8) with logs & metrics.

        match offset.try_into() {
            Ok(offset) => self.bus_write(offset, data),
            Err(_) => debug!("Invalid serial console write offset."),
        }
    }
}
/// Errors encountered during device operation.
#[derive(Debug)]
pub enum Error {
    /// Failed to create an event manager for device events.
    EventManager(event_manager::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy::EventFdTrigger;

    use std::io::sink;

    fn check_invalid_data(invalid_data: &mut [u8]) {
        let interrupt_evt = EventFdTrigger::new(libc::EFD_NONBLOCK).unwrap();
        let mut serial_console = SerialWrapper(Serial::new(interrupt_evt, sink()));
        let valid_iir_offset = 2;

        // Check that passing invalid data does not result in a crash.
        #[cfg(target_arch = "x86_64")]
        serial_console.pio_read(PioAddress(0), valid_iir_offset, invalid_data);
        #[cfg(target_arch = "aarch64")]
        serial_console.mmio_read(MmioAddress(0), valid_iir_offset, invalid_data);

        // The same scenario happens for writes.
        #[cfg(target_arch = "x86_64")]
        serial_console.pio_write(PioAddress(0), valid_iir_offset, invalid_data);
        #[cfg(target_arch = "aarch64")]
        serial_console.mmio_write(MmioAddress(0), valid_iir_offset, invalid_data);
    }

    #[test]
    fn test_empty_data() {
        check_invalid_data(&mut []);
    }

    #[test]
    fn test_longer_data() {
        check_invalid_data(&mut [0; 2]);
    }

    #[test]
    fn test_invalid_offset() {
        let interrupt_evt = EventFdTrigger::new(libc::EFD_NONBLOCK).unwrap();
        let mut serial_console = SerialWrapper(Serial::new(interrupt_evt, sink()));
        let data = [0];

        // Check that passing an invalid offset does not result in a crash.
        #[cfg(target_arch = "x86_64")]
        {
            let invalid_offset = PioAddressOffset::MAX;
            serial_console.pio_write(PioAddress(0), invalid_offset, &data);
        }
        #[cfg(target_arch = "aarch64")]
        {
            let invalid_offset = u64::MAX;
            serial_console.mmio_write(MmioAddress(0), invalid_offset, &data);
        }
    }

    #[test]
    fn test_valid_write_and_read() {
        let interrupt_evt = EventFdTrigger::new(libc::EFD_NONBLOCK).unwrap();
        let mut serial_console = SerialWrapper(Serial::new(interrupt_evt, sink()));
        let write_data = [5];
        let mut read_data = [0];
        let offset = 7;

        // Write on and read from the serial console.
        #[cfg(target_arch = "x86_64")]
        {
            serial_console.pio_write(PioAddress(0), offset, &write_data);
            serial_console.pio_read(PioAddress(0), offset, read_data.as_mut());
        }
        #[cfg(target_arch = "aarch64")]
        {
            serial_console.mmio_write(MmioAddress(0), offset, &write_data);
            serial_console.mmio_read(MmioAddress(0), offset, read_data.as_mut());
        }

        assert_eq!(&write_data, &read_data);
    }
}
