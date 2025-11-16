// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{aml_as_bytes, u8sum, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;

// SRAT is the place where proximity domains are defined, and _PXM
// (found in DSDT and/or SSDT) provides a mechanism to associate a
// device object (and its children) to an SRAT-defined proximity
// domain.
pub struct SRAT {
    header: TableHeader,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
}

impl SRAT {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"SRAT",
            // 12 reserved bytes
            length: (TableHeader::len() as u32 + 12).into(),
            revision: 1,
            checksum: 0,
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
        };

        let mut cksum = Checksum::default();
        cksum.append(header.as_bytes());
        cksum.add(1); // from the reserved `1` immediately after the header
        header.checksum = cksum.value();

        Self {
            header,
            checksum: cksum,
            structures: Vec::new(),
        }
    }

    fn update_header(&mut self, len: u32, sum: u8) {
        let old_len = self.header.length.get();
        let new_len = len + old_len;
        self.header.length.set(new_len);

        // Remove the bytes from the old length, add the new length
        // and the new data.
        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.add(sum);
        self.header.checksum = self.checksum.value();
    }

    pub fn add_memory_affinity(&mut self, st: MemoryAffinity) {
        self.update_header(MemoryAffinity::len() as u32, st.u8sum());
        self.structures.push(Box::new(st));
    }

    pub fn add_generic_initiator(&mut self, st: GenericInitiator) {
        self.update_header(GenericInitiator::len() as u32, st.u8sum());
        self.structures.push(Box::new(st));
    }

    pub fn add_rintc_affinity(&mut self, ra: RintcAffinity) {
        self.update_header(RintcAffinity::len() as u32, ra.u8sum());
        self.structures.push(Box::new(ra));
    }
}

impl Aml for SRAT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        sink.dword(1); // reserved to be 1 for backward compatibility.
        sink.qword(0); // reserved

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

#[repr(u8)]
enum SratStructureType {
    MemoryAffinity = 1,
    GenericInitiator = 5,
    RintcAffinity = 7,
}

#[repr(u32)]
enum MemoryAffinityFlags {
    Enabled = 1 << 0,
    HotPluggable = 1 << 1,
    NonVolatile = 1 << 2,
}

pub struct MemoryAffinity {
    proximity_domain: u32,
    base_address: u64,
    length: u64,
    flags: u32,
}

impl MemoryAffinity {
    pub fn new(proximity_domain: u32, base_address: u64, length: u64) -> Self {
        Self {
            proximity_domain,
            base_address,
            length,
            flags: 0,
        }
    }

    pub fn enabled(mut self) -> Self {
        self.flags |= MemoryAffinityFlags::Enabled as u32;
        self
    }

    pub fn hotpluggable(mut self) -> Self {
        self.flags |= MemoryAffinityFlags::HotPluggable as u32;
        self
    }

    pub fn nonvolatile(mut self) -> Self {
        self.flags |= MemoryAffinityFlags::NonVolatile as u32;
        self
    }

    fn len() -> usize {
        40
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for MemoryAffinity {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(SratStructureType::MemoryAffinity as u8);
        sink.byte(Self::len() as u8);
        sink.dword(self.proximity_domain);
        sink.word(0); // reserved
        sink.dword((self.base_address & 0xffff_ffff) as u32);
        sink.dword(((self.base_address >> 32) & 0xffff_ffff) as u32);
        sink.dword((self.length & 0xffff_ffff) as u32);
        sink.dword(((self.length >> 32) & 0xffff_ffff) as u32);
        sink.dword(0); // reserved
        sink.dword(self.flags);
        sink.qword(0); // reserved
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Handle {
    Acpi {
        hid: [u8; 8],
        uid: [u8; 4],
    },
    Pci {
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
    },
}

impl Handle {
    pub fn new_acpi(hid: [u8; 8], uid: [u8; 4]) -> Self {
        Handle::Acpi { hid, uid }
    }

    /// Segment must be 0 for systems with fewer than 255 buses
    pub fn new_pci(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        assert!(device < 32);
        assert!(function < 8);

        Handle::Pci {
            segment,
            bus,
            device,
            function,
        }
    }

    fn devfn(device: u8, function: u8) -> u8 {
        device << 3 | function
    }
}

impl Aml for Handle {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        match self {
            Handle::Acpi { hid, uid } => {
                for byte in hid {
                    sink.byte(*byte);
                }
                for byte in uid {
                    sink.byte(*byte);
                }

                sink.dword(0); // reserved
            }
            Handle::Pci {
                segment,
                bus,
                device,
                function,
            } => {
                sink.word(*segment);
                sink.byte(*bus);
                sink.byte(Self::devfn(*device, *function));

                // 12 reserved bytes
                sink.dword(0);
                sink.qword(0);
            }
        }
    }
}

#[repr(u8)]
enum GenericInitiatorFlags {
    Enabled = 1 << 0,
    ArchitecturalTransactions = 1 << 1,
}

pub struct GenericInitiator {
    proximity_domain: u32,
    handle: Handle,
    flags: u32,
}

impl GenericInitiator {
    pub fn new(proximity_domain: u32, handle: Handle) -> Self {
        Self {
            proximity_domain,
            handle,
            flags: 0,
        }
    }

    pub fn enabled(mut self) -> Self {
        self.flags |= GenericInitiatorFlags::Enabled as u32;
        self
    }

    pub fn architectural(mut self) -> Self {
        self.flags |= GenericInitiatorFlags::ArchitecturalTransactions as u32;
        self
    }

    fn len() -> usize {
        32
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for GenericInitiator {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(SratStructureType::GenericInitiator as u8);
        sink.byte(Self::len() as u8);
        sink.byte(0); // reserved
        sink.byte(match self.handle {
            Handle::Acpi { .. } => 0,
            Handle::Pci { .. } => 1,
        });
        sink.dword(self.proximity_domain);
        self.handle.to_aml_bytes(sink);
        sink.dword(self.flags);
        sink.dword(0); // reserved
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct RintcAffinity {
    r#type: u8,
    length: u8,
    reserved: U16,
    acpi_processor_uid: [u8; 4],
    flags: U32,
    clock_domain: U32,
}

impl RintcAffinity {
    const FLAGS_ENABLED: u32 = 1 << 0;

    fn len() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn new(acpi_processor_uid: [u8; 4], clock_domain: u32) -> Self {
        Self {
            r#type: SratStructureType::RintcAffinity as u8,
            length: 20,
            reserved: 0.into(),
            acpi_processor_uid,
            flags: 0.into(),
            clock_domain: clock_domain.into(),
        }
    }

    pub fn enabled(mut self) -> Self {
        self.flags = (self.flags.get() | Self::FLAGS_ENABLED).into();
        self
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

aml_as_bytes!(RintcAffinity);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_srat() {
        let srat = SRAT::new(*b"FOOBAR", *b"DECAFCOF", 0xdead_beef);

        let mut bytes = Vec::new();
        srat.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_memory_affinity_entries() {
        let mut srat = SRAT::new(*b"FOOBAR", *b"DECAFCOF", 0xdead_beef);

        srat.add_memory_affinity(MemoryAffinity::new(0x42, 0x1_2030_4050, 0x8060_8010).enabled());
        srat.add_memory_affinity(
            MemoryAffinity::new(0x27, 0x8_6007_8009, 0x1_2005_4003)
                .enabled()
                .hotpluggable(),
        );

        let mut bytes = Vec::new();
        srat.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_generic_initiator_entries() {
        let mut srat = SRAT::new(*b"FOOBAR", *b"DECAFCOF", 0xdead_beef);

        srat.add_generic_initiator(
            GenericInitiator::new(
                0x42,
                Handle::Acpi {
                    hid: *b"ABCD____",
                    uid: [1, 2, 3, 4],
                },
            )
            .enabled()
            .architectural(),
        );
        srat.add_generic_initiator(
            GenericInitiator::new(
                0xff,
                Handle::Pci {
                    segment: 1,
                    bus: 2,
                    device: 3,
                    function: 4,
                },
            )
            .enabled(),
        );

        let mut bytes = Vec::new();
        srat.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_rintc_affinity() {
        let mut srat = SRAT::new(*b"FOOBAR", *b"SRATSRAT", 0xdead_beef);
        srat.add_rintc_affinity(RintcAffinity::new([0x42, 0x37, 0x58, 0xde], 0xde583742).enabled());

        let mut bytes = Vec::new();
        srat.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}