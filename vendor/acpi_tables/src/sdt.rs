// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0
//

extern crate alloc;

use crate::{Aml, AmlSink};
use alloc::vec::Vec;
use zerocopy::AsBytes;

#[repr(packed)]
#[derive(Clone, Copy, AsBytes)]
pub struct GenericAddress {
    pub address_space_id: u8,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

impl GenericAddress {
    fn access_size_of<T>() -> u8 {
        // https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/05_ACPI_Software_Programming_Model/ACPI_Software_Programming_Model.html#generic-address-structure-gas
        match core::mem::size_of::<T>() {
            1 => 1,
            2 => 2,
            4 => 3,
            8 => 4,
            _ => unreachable!(),
        }
    }

    pub fn io_port_address<T>(address: u16) -> Self {
        GenericAddress {
            address_space_id: 1,
            register_bit_width: 8 * core::mem::size_of::<T>() as u8,
            register_bit_offset: 0,
            access_size: Self::access_size_of::<T>(),
            address: u64::from(address),
        }
    }
    pub fn mmio_address<T>(address: u64) -> Self {
        GenericAddress {
            address_space_id: 0,
            register_bit_width: 8 * core::mem::size_of::<T>() as u8,
            register_bit_offset: 0,
            access_size: Self::access_size_of::<T>(),
            address,
        }
    }
}

pub struct Sdt {
    data: Vec<u8>,
}

impl AmlSink for Sdt {
    fn byte(&mut self, byte: u8) {
        self.append(byte);
    }
}

impl Sdt {
    pub fn new(
        signature: [u8; 4],
        length: u32,
        revision: u8,
        oem_id: [u8; 6],
        oem_table: [u8; 8],
        oem_revision: u32,
    ) -> Self {
        assert!(length >= 36);

        let mut data = Vec::with_capacity(length as usize);
        data.extend_from_slice(&signature);
        data.extend_from_slice(&length.to_le_bytes());
        data.push(revision);
        data.push(0); // checksum
        data.extend_from_slice(&oem_id);
        data.extend_from_slice(&oem_table);
        data.extend_from_slice(&oem_revision.to_le_bytes());
        data.extend_from_slice(&crate::CREATOR_ID);
        data.extend_from_slice(&crate::CREATOR_REVISION);
        assert_eq!(data.len(), 36);

        data.resize(length as usize, 0);
        let mut sdt = Sdt { data };

        sdt.update_checksum();
        sdt
    }

    pub fn update_checksum(&mut self) {
        self.data[9] = 0;
        let checksum = super::generate_checksum(self.data.as_slice());
        self.data[9] = checksum
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn append<T: AsBytes>(&mut self, value: T) {
        let orig_length = self.data.len();
        let new_length = orig_length + core::mem::size_of::<T>();
        self.data.resize(new_length, 0);
        self.write_u32(4, new_length as u32);
        self.write(orig_length, value);
    }

    pub fn append_slice(&mut self, data: &[u8]) {
        let orig_length = self.data.len();
        let new_length = orig_length + data.len();
        self.write_u32(4, new_length as u32);
        self.data.extend_from_slice(data);
        self.update_checksum();
    }

    /// Write a slice of data at a specific offset
    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) {
        assert!(offset + data.len() <= self.data.len());
        self.data.as_mut_slice()[offset..offset + data.len()].copy_from_slice(data);
        self.update_checksum();
    }

    /// Write a value at the given offset
    pub fn write<T: AsBytes>(&mut self, offset: usize, value: T) {
        self.write_bytes(offset, value.as_bytes())
    }

    pub fn write_u8(&mut self, offset: usize, val: u8) {
        self.write(offset, val);
    }

    pub fn write_u16(&mut self, offset: usize, val: u16) {
        self.write(offset, val);
    }

    pub fn write_u32(&mut self, offset: usize, val: u32) {
        self.write(offset, val);
    }

    pub fn write_u64(&mut self, offset: usize, val: u64) {
        self.write(offset, val);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Aml for Sdt {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.vec(&self.data);
    }
}

#[cfg(test)]
mod tests {
    use super::{GenericAddress, Sdt};

    #[test]
    fn test_sdt() {
        let mut sdt = Sdt::new(*b"TEST", 40, 1, *b"CLOUDH", *b"TESTTEST", 1);
        let sum: u8 = sdt
            .as_slice()
            .iter()
            .fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        sdt.write_u32(36, 0x12345678);
        let sum: u8 = sdt
            .as_slice()
            .iter()
            .fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_generic_address_access_size() {
        let byte_mmio = GenericAddress::mmio_address::<u8>(0x1000);
        assert_eq!(byte_mmio.access_size, 1);
        let word_mmio = GenericAddress::mmio_address::<u16>(0x1000);
        assert_eq!(word_mmio.access_size, 2);
        let dword_mmio = GenericAddress::mmio_address::<u32>(0x1000);
        assert_eq!(dword_mmio.access_size, 3);
        let qword_mmio = GenericAddress::mmio_address::<u64>(0x1000);
        assert_eq!(qword_mmio.access_size, 4);
    }
}