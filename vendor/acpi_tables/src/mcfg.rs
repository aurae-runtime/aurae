// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

extern crate alloc;

use crate::{Aml, AmlSink, Checksum, TableHeader};
use alloc::vec::Vec;
use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U16 = byteorder::U16<LE>;
type U64 = byteorder::U64<LE>;

pub struct MCFG {
    header: TableHeader,
    checksum: Checksum,
    entries: Vec<EcamEntry>,
}

impl MCFG {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"MCFG",
            length: ((TableHeader::len() + 8) as u32).into(),
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

    fn update_header(&mut self, data: &[u8]) {
        let len = data.len() as u32;
        let old_len = self.header.length.get();
        let new_len = len + old_len;
        self.header.length.set(new_len);

        // Remove the bytes from the old length, add the new length
        // and the new data.
        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.append(data);
        self.header.checksum = self.checksum.value();
    }

    pub fn add_ecam(&mut self, base_addr: u64, segment: u16, start_bus: u8, end_bus: u8) {
        let entry = EcamEntry {
            base_addr: base_addr.into(),
            segment: segment.into(),
            start_bus,
            end_bus,
            _reserved: [0, 0, 0, 0],
        };

        self.update_header(entry.as_bytes());
        self.entries.push(entry);
    }
}

impl Aml for MCFG {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        // 8 reserved bytes
        sink.qword(0);

        for entry in &self.entries {
            entry.to_aml_bytes(sink);
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, AsBytes)]
struct EcamEntry {
    base_addr: U64,
    segment: U16,
    start_bus: u8,
    end_bus: u8,
    _reserved: [u8; 4],
}

#[cfg(test)]
impl EcamEntry {
    fn len() -> usize {
        core::mem::size_of::<EcamEntry>()
    }
}

crate::aml_as_bytes!(EcamEntry);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcfg() {
        let mut mcfg = MCFG::new([1, 2, 3, 4, 5, 6], [1, 2, 3, 4, 5, 6, 7, 8], 1234);

        let mut bytes = Vec::new();
        mcfg.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(TableHeader::len() + 8, bytes.len());

        mcfg.add_ecam(0xc000_0000, 42, 0, 0x20);
        let mut bytes = Vec::new();
        mcfg.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(TableHeader::len() + 8 + EcamEntry::len(), bytes.len());

        mcfg.add_ecam(0x1234_5678, 3920, 5, 0xfe);
        let mut bytes = Vec::new();
        mcfg.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);

        assert_eq!(TableHeader::len() + 8 + EcamEntry::len() * 2, bytes.len());
    }
}