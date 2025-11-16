// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::AsBytes;

extern crate alloc;
use alloc::vec::Vec;

use crate::{Aml, AmlSink, Checksum, TableHeader};

pub struct SLIT {
    header: TableHeader,
    checksum: Checksum,
    entries: Vec<u8>,
    localities: u32,
}

pub const UNREACHABLE_LOCALITY: u8 = 0xff;

impl SLIT {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32, localities: u32) -> Self {
        let entry_count = localities * localities;
        let length: u32 =
            TableHeader::len() as u32 + entry_count + core::mem::size_of::<u64>() as u32;

        let mut header = TableHeader {
            signature: *b"SLIT",
            length: length.into(),
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
        cksum.append((localities as u64).as_bytes());

        // Default value for one node to itself is 10
        let mut entries = Vec::with_capacity(entry_count as usize);
        if entry_count > 0 {
            entries.resize(entry_count as usize, 10);
            cksum.append(&entries);
        }

        header.checksum = cksum.value();
        Self {
            header,
            checksum: cksum,
            entries,
            localities,
        }
    }

    fn update_header(&mut self, old_values: &[u8], new_value: u8) {
        self.checksum.delete(old_values);
        self.checksum.append(&[new_value, new_value]);
        self.header.checksum = self.checksum.value();
    }

    /// Set the relative locality distance between two domains
    /// (10-254, 10 is the value from one node to itself).
    pub fn set_distance(&mut self, domain_a: usize, domain_b: usize, locality_value: u8) {
        let old_values = [
            self.entries[domain_a + self.localities as usize * domain_b],
            self.entries[domain_b + self.localities as usize * domain_a],
        ];

        self.entries[domain_a + self.localities as usize * domain_b] = locality_value;
        self.entries[domain_b + self.localities as usize * domain_a] = locality_value;
        self.update_header(&old_values, locality_value);
    }
}

impl Aml for SLIT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        sink.qword(self.localities as u64);
        for entry in &self.entries {
            sink.byte(*entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slit() {
        let slit = SLIT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
            0,
        );

        let mut bytes = Vec::new();
        slit.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_slit_entries() {
        let mut slit = SLIT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
            2,
        );
        let mut bytes = Vec::new();
        slit.set_distance(0, 1, 15);
        slit.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_slit_entries_more() {
        let mut slit = SLIT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
            4,
        );
        let mut bytes = Vec::new();
        slit.set_distance(0, 1, 15);
        slit.set_distance(0, 2, UNREACHABLE_LOCALITY);
        slit.set_distance(0, 3, 20);
        slit.set_distance(1, 2, 25);
        slit.set_distance(1, 3, 30);
        slit.set_distance(2, 3, 35);

        slit.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}