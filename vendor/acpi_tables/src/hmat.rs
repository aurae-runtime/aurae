// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec};

use crate::{aml_as_bytes, assert_same_size, u8sum, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;

pub struct HMAT {
    header: TableHeader,
    checksum: Checksum,
    entries: Vec<Box<dyn Aml>>,
}

#[repr(u16)]
enum HmatStructureType {
    MemoryProximityDomain = 0,
    SystemLocality = 1,
    MemorySideCache = 2,
}

impl HMAT {
    fn header_len() -> usize {
        let reserved = 4;
        TableHeader::len() + reserved
    }

    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"HMAT",
            length: (Self::header_len() as u32).into(),
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
        header.checksum = cksum.value();
        Self {
            header,
            checksum: cksum,
            entries: Vec::new(),
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

    pub fn add_memory_proximity(&mut self, m: MemoryProximityDomain) {
        self.update_header(MemoryProximityDomain::len() as u32, u8sum(&m));
        self.entries.push(Box::new(m));
    }

    pub fn add_system_locality(&mut self, s: SystemLocality) {
        self.update_header(s.len() as u32, u8sum(&s));
        self.entries.push(Box::new(s));
    }

    pub fn add_memory_side_cache(&mut self, c: MemorySideCache) {
        self.update_header(c.len() as u32, u8sum(&c));
        self.entries.push(Box::new(c));
    }
}

impl Aml for HMAT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        self.header.to_aml_bytes(sink);

        // 4 reserved bytes
        sink.dword(0);

        for entry in &self.entries {
            entry.to_aml_bytes(sink);
        }
    }
}

// This structure describes the system physical address range occupied
// by the memory subsystem and its associativity with a processor
// proximity domain.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct MemoryProximityDomain {
    r#type: U16,
    _reserved0: U16,
    length: U32,
    flags: U16,
    _reserved1: U16,
    proximity_domain_initiator: U32,
    proximity_domain_memory: U32,
    _reserved2: [u8; 20], // Reserved data and deprecated fields
}

assert_same_size!(MemoryProximityDomain, [u8; 40]);

impl MemoryProximityDomain {
    const DOMAIN_VALID: u16 = 1 << 0;

    fn len() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn new(proximity_domain_initiator: u32, proximity_domain_memory: u32) -> Self {
        Self {
            r#type: (HmatStructureType::MemoryProximityDomain as u16).into(),
            length: (Self::len() as u32).into(),
            flags: Self::DOMAIN_VALID.into(),
            proximity_domain_initiator: proximity_domain_initiator.into(),
            proximity_domain_memory: proximity_domain_memory.into(),
            ..Default::default()
        }
    }
}

aml_as_bytes!(MemoryProximityDomain);

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DataType {
    // if read/write latencies are identical
    AccessLatency = 0,
    ReadLatency = 1,
    WriteLatency = 2,
    // if read/write bandwidths are identical
    AccessBandwidth = 3,
    ReadBandwidth = 4,
    WriteBandwidth = 5,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum MinTransferSize {
    SizeByteAligned = 0,
    Size64b = 1,
    Size128b = 2,
    Size256b = 3,
    Size512b = 4,
    Size1k = 5,
    Size2k = 6,
    Size4k = 7,
    Size8k = 8,
    Size16k = 9,
    Size32k = 10,
    Size64k = 11,
}

pub struct SystemLocality {
    flags: u8,
    data_type: DataType,
    min_transfer_size: MinTransferSize,
    entry_base_unit: u64,
    initiators: Vec<u32>,
    targets: Vec<u32>,
    entries: Vec<u16>,
}

#[repr(u8)]
pub enum LocalityType {
    Memory = 0,
    FirstLevelCache = 1,
    SecondLevelCache = 2,
    ThirdLevelCache = 3,
}

// This structure provides a matrix that describes the normalized
// memory read/write latency or the read/write bandwidth between
// initiator proximity domains (processors or DMA) and target
// proximity domains (memory). The entry base unit for latency
// is in picoseconds whereas for bandwidth it is MB/s.
impl SystemLocality {
    const MINIMUM_TRANSFER_SIZE_REQD: u8 = 0x10;
    const NON_SEQUENTIAL_TRANSFERS: u8 = 0x20;

    const UNREACHABLE_DOMAIN: u16 = 0xffff;

    pub fn new(
        loc_type: LocalityType,
        data_type: DataType,
        min_transfer_size: MinTransferSize,
        entry_base_unit: u64,
        num_initiators: usize,
        num_targets: usize,
    ) -> Self {
        let initiators = vec![0; num_initiators];
        let targets = vec![0; num_targets];
        let entries = vec![Self::UNREACHABLE_DOMAIN; num_initiators * num_targets];

        Self {
            flags: loc_type as u8,
            data_type,
            min_transfer_size,
            entry_base_unit,
            initiators,
            targets,
            entries,
        }
    }

    pub fn non_sequential_transfers(&mut self) {
        self.flags |= Self::NON_SEQUENTIAL_TRANSFERS;
    }

    pub fn minimum_transfer_size_required(&mut self) {
        self.flags |= Self::MINIMUM_TRANSFER_SIZE_REQD;
    }

    pub fn set_initiator_value(&mut self, idx: usize, value: u32) {
        self.initiators[idx] = value;
    }

    pub fn set_target_value(&mut self, idx: usize, value: u32) {
        self.targets[idx] = value;
    }

    pub fn set_entry_value(&mut self, initiator_idx: usize, target_idx: usize, value: u16) {
        self.entries[initiator_idx * self.initiators.len() + target_idx] = value;
    }

    fn len(&self) -> usize {
        core::mem::size_of::<u32>() * self.initiators.len()
            + core::mem::size_of::<u32>() * self.targets.len()
            + core::mem::size_of::<u16>() * self.entries.len()
            + 32
    }
}

impl Aml for SystemLocality {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.word(HmatStructureType::SystemLocality as u16);
        sink.word(0); // reserved
        sink.dword(self.len() as u32);
        sink.byte(self.flags);
        sink.byte(self.data_type as u8);
        sink.byte(self.min_transfer_size as u8);
        sink.byte(0); // reserved
        sink.dword(self.initiators.len() as u32);
        sink.dword(self.targets.len() as u32);
        sink.dword(0); // reserved
        sink.qword(self.entry_base_unit);

        for initiator in &self.initiators {
            sink.dword(*initiator);
        }

        for target in &self.targets {
            sink.dword(*target);
        }

        for entry in &self.entries {
            sink.word(*entry);
        }
    }
}

pub struct MemorySideCache {
    // This number should match the corresponding entry in the SRAT's
    // Memory Affinity Structure
    proximity_domain: u32,
    cache_size: u64,
    attributes: u32,
    smbios_handles: Vec<u16>,
}

#[repr(u32)]
pub enum CacheLevel {
    None = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[repr(u32)]
pub enum Associativity {
    None = 0,
    DirectMapped = 1,
    Complex = 2,
}

#[repr(u32)]
pub enum WritePolicy {
    None = 0,
    Writeback = 1,
    Writethrough = 2,
}

impl MemorySideCache {
    pub fn new(
        proximity_domain: u32,
        cache_size: u64,
        total_cache_levels: CacheLevel,
        this_cache_level: CacheLevel,
        associativity: Associativity,
        write_policy: WritePolicy,
        cacheline_size: u16,
    ) -> Self {
        let attributes = total_cache_levels as u32
            | (this_cache_level as u32) << 4
            | (associativity as u32) << 8
            | (write_policy as u32) << 12
            | (cacheline_size as u32) << 16;
        Self {
            proximity_domain,
            cache_size,
            attributes,
            smbios_handles: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        32 + self.smbios_handles.len() * core::mem::size_of::<u16>()
    }

    pub fn add_smbios_handle(&mut self, handle: u16) {
        self.smbios_handles.push(handle);
    }
}

impl Aml for MemorySideCache {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.word(HmatStructureType::MemorySideCache as u16);
        sink.word(0); // reserved
        sink.dword(self.len() as u32);
        sink.dword(self.proximity_domain);
        sink.dword(0); // reserved
        sink.qword(self.cache_size);
        sink.dword(self.attributes);
        sink.word(0); // reserved
        sink.word(self.smbios_handles.len() as u16);
        for handle in &self.smbios_handles {
            sink.word(*handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_checksum(hmat: &HMAT) {
        let mut bytes = Vec::new();
        hmat.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    fn get_size(hmat: &HMAT) -> usize {
        let mut bytes = Vec::new();
        hmat.to_aml_bytes(&mut bytes);
        bytes.len()
    }

    #[test]
    fn test_empty() {
        let hmat = HMAT::new(*b"TEST__", *b"TESTTEST", 0x4242_4242);
        check_checksum(&hmat);
    }

    #[test]
    fn test_mem_proximity() {
        let mut hmat = HMAT::new(*b"TEST__", *b"TESTTEST", 0x4242_4242);

        let m = MemoryProximityDomain::new(0x42, 0x37);
        hmat.add_memory_proximity(m);
        check_checksum(&hmat);

        assert_eq!(
            get_size(&hmat),
            HMAT::header_len() + MemoryProximityDomain::len()
        );
    }

    #[test]
    fn test_system_locality() {
        let mut hmat = HMAT::new(*b"TEST__", *b"TESTTEST", 0x4242_4242);

        let s = {
            let mut s = SystemLocality::new(
                LocalityType::Memory,
                DataType::AccessLatency,
                MinTransferSize::Size4k,
                // since this is a latency entry, the entry base unit
                // is in picoseconds, so 1000 is 1 nanosecond
                1000,
                3, // 3 initiators
                2, // 2 targets
            );
            s.set_initiator_value(0, 0x100);
            s.set_initiator_value(1, 0x101);
            s.set_initiator_value(2, 0x102);
            s.set_target_value(0, 0x1001);
            s.set_target_value(1, 0x1001);
            s.set_entry_value(0, 1, 10); // 10 nanosecond access latency from initiator 0 to target 1

            let mut v = Vec::new();
            s.to_aml_bytes(&mut v);
            assert_eq!(s.len(), v.len());
            s
        };

        let s_len = s.len();
        hmat.add_system_locality(s);
        check_checksum(&hmat);
        assert_eq!(get_size(&hmat), HMAT::header_len() + s_len);
    }

    #[test]
    fn test_memory_side_cache() {
        let mut hmat = HMAT::new(*b"TEST__", *b"TESTTEST", 0x4242_4242);

        let mut c = MemorySideCache::new(
            0x1234_5678,
            0x90ab_cdef_1234_5678,
            CacheLevel::Three,
            CacheLevel::Two,
            Associativity::DirectMapped,
            WritePolicy::Writeback,
            64,
        );
        c.add_smbios_handle(42);
        c.add_smbios_handle(37);
        c.add_smbios_handle(65535);

        let c_len = c.len();
        hmat.add_memory_side_cache(c);
        check_checksum(&hmat);

        assert_eq!(get_size(&hmat), HMAT::header_len() + c_len);
    }
}