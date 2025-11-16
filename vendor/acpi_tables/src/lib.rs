// Copyright © 2019 Intel Corporation
// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

#![crate_type = "staticlib"]
#![no_std]

//! ACPI table generation.

pub mod aml;
pub mod bert;
pub mod cedt;
pub mod facs;
pub mod fadt;
pub mod gas;
pub mod hest;
pub mod hmat;
pub mod madt;
pub mod mcfg;
pub mod pptt;
pub mod rhct;
pub mod rimt;
pub mod rqsc;
pub mod rsdp;
pub mod sdt;
pub mod slit;
pub mod spcr;
pub mod srat;
pub mod tpm2;
pub mod viot;
pub mod xsdt;

extern crate alloc;

use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U32 = byteorder::U32<LE>;

// Rust-VMM ACPI Tables
pub const CREATOR_ID: [u8; 4] = *b"RVAT";
pub const CREATOR_REVISION: [u8; 4] = [0, 0, 0, 1];

/// This trait is used by the `Aml` trait as a sink for the actual
/// bytecode. An application using this library must provide a type
/// that implements this trait to receive the bytecode.
pub trait AmlSink {
    fn byte(&mut self, byte: u8);

    fn word(&mut self, word: u16) {
        for byte in word.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn dword(&mut self, dword: u32) {
        for byte in dword.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn qword(&mut self, qword: u64) {
        for byte in qword.to_le_bytes() {
            self.byte(byte);
        }
    }

    fn vec(&mut self, v: &[u8]) {
        for byte in v {
            self.byte(*byte);
        }
    }
}

/// The trait Aml can be implemented by ACPI objects or ACPI tables to
/// translate itself into the AML raw data.
pub trait Aml {
    /// Serialize an ACPI object into AML bytecode using the provided
    /// AmlSink object.
    /// * `sink` - The sink used to receive the AML bytecode.
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink);
}

/// Simplify the library by treating Vec<u8> as a valid AmlSink.
impl AmlSink for alloc::vec::Vec<u8> {
    fn byte(&mut self, byte: u8) {
        self.push(byte);
    }
}

/// Standard header for many ACPI tables
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
struct TableHeader {
    pub signature: [u8; 4],
    pub length: U32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    // Note: oem_table_id must match the OEM Table ID in the RSDT
    pub oem_table_id: [u8; 8],
    pub oem_revision: U32,
    pub creator_id: [u8; 4],
    pub creator_revision: [u8; 4],
}

impl TableHeader {
    pub fn len() -> usize {
        core::mem::size_of::<TableHeader>()
    }
}

aml_as_bytes!(TableHeader);

/// Object used to keep track of a rolling u8 sum, for which
/// the checksum can be derived via value().
#[derive(Debug, Default)]
pub struct Checksum {
    value: u8,
}

impl AmlSink for Checksum {
    fn byte(&mut self, byte: u8) {
        self.add(byte);
    }
}

impl Checksum {
    pub fn append(&mut self, data: &[u8]) {
        let mut value: u8 = self.value;
        for b in data {
            value = value.wrapping_add(*b);
        }

        self.value = value;
    }

    pub fn delete(&mut self, data: &[u8]) {
        let mut value: u8 = self.value;
        for b in data {
            value = value.wrapping_sub(*b);
        }

        self.value = value;
    }

    pub fn add(&mut self, data: u8) {
        self.value = self.value.wrapping_add(data);
    }

    pub fn sub(&mut self, data: u8) {
        self.value = self.value.wrapping_sub(data);
    }

    pub fn raw_value(&self) -> u8 {
        self.value
    }

    pub fn value(&self) -> u8 {
        (255 - self.value).wrapping_add(1)
    }
}

pub fn u8sum(aml: &dyn Aml) -> u8 {
    let mut cksum = Checksum::default();
    aml.to_aml_bytes(&mut cksum);
    cksum.raw_value()
}

/// Generate an 8-bit checksum over a byte slice.
fn generate_checksum(data: &[u8]) -> u8 {
    (255 - data.iter().fold(0u8, |acc, x| acc.wrapping_add(*x))).wrapping_add(1)
}

#[macro_export]
macro_rules! assert_same_size {
    ($x:ty, $y:ty) => {
        const _: fn() = || {
            let _ = core::mem::transmute::<$x, $y>;
        };
    };
}

#[macro_export]
macro_rules! aml_as_bytes {
    ($x:ty) => {
        impl Aml for $x {
            fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
                for byte in self.as_bytes() {
                    sink.byte(*byte);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! mutable_setter {
    ($name:ident, $x:ty) => {
        pub fn $name(mut self, $name: $x) -> Self {
            self.$name = $name.into();
            self
        }
    };
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let mut c = Checksum::default();
        assert!(c.value() == 0);
        c.add(1);
        assert!(c.value() == 255);
        c.add(1);
        assert!(c.value() == 254);
        c.add(255);
        c.add(1);
        assert!(c.value() == 254);
        c.append(&[255]);
        assert!(c.value() == 255);
        c.append(&[128, 128]);
        assert!(c.value() == 255);
        c.add(1);
        assert!(c.value() == 254);
    }
}