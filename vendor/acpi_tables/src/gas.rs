// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use crate::{Aml, AmlSink};
use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U64 = byteorder::U64<LE>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, AsBytes, Default)]
pub enum AddressSpace {
    #[default]
    SystemMemory = 0x0,
    SystemIo = 0x1,
    PciConfigSpace = 0x2,
    EmbeddedController = 0x3,
    Smbus = 0x4,
    SystemCmos = 0x5,
    PciBarTarget = 0x6,
    Ipmi = 0x7,
    GeneralPursposeIo = 0x8,
    GenericSerialBus = 0x9,
    PlatformCommunicationsChannel = 0xa,
    PlatformRuntimeMechanism = 0xb,
    FunctionalFixedHardware = 0x7f,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, AsBytes, Default)]
pub enum AccessSize {
    #[default]
    Undefined = 0,
    ByteAccess = 1,
    WordAccess = 2,
    DwordAccess = 3,
    QwordAccess = 4,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, AsBytes, Default)]
pub struct GAS {
    pub address_space_id: AddressSpace,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: AccessSize,
    pub address: U64,
}

impl GAS {
    pub fn len() -> usize {
        core::mem::size_of::<GAS>()
    }

    pub fn new(
        address_space_id: AddressSpace,
        register_bit_width: u8,
        register_bit_offset: u8,
        access_size: AccessSize,
        address: u64,
    ) -> Self {
        Self {
            address_space_id,
            register_bit_width,
            register_bit_offset,
            access_size,
            address: address.into(),
        }
    }

    // NOTE: PCI Configuration space addresses must be confined to devices on
    // PCI Segment Group 0, bus 0.
    pub fn new_pci_config(
        register_bit_width: u8,
        access_size: AccessSize,
        device: u8,
        function: u8,
        register: u16,
    ) -> Self {
        let address = ((device as u64) << 32 | (function as u64) << 16 | (register as u64)).into();
        Self {
            address_space_id: AddressSpace::PciConfigSpace,
            register_bit_width,
            register_bit_offset: 0,
            access_size,
            address,
        }
    }
}

crate::assert_same_size!(GAS, [u8; 12]);

impl Aml for GAS {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(self.address_space_id as u8);
        sink.byte(self.register_bit_width);
        sink.byte(self.register_bit_offset);
        sink.byte(self.access_size as u8);
        sink.qword(self.address.into());
    }
}