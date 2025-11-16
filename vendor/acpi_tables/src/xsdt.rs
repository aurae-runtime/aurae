// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

extern crate alloc;

use crate::{Aml, AmlSink, Checksum, TableHeader};
use alloc::vec::Vec;
use zerocopy::AsBytes;

pub struct XSDT {
    header: TableHeader,
    checksum: Checksum,
    entries: Vec<u64>,
}

impl XSDT {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"XSDT",
            length: (TableHeader::len() as u32).into(),
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

    pub fn add_entry(&mut self, entry: u64) {
        let old_len = self.header.length.get();
        let new_len = old_len + core::mem::size_of::<u64>() as u32;
        self.header.length.set(new_len);

        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.append(&entry.to_le_bytes());
        self.header.checksum = self.checksum.value();
        self.entries.push(entry);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        TableHeader::len() + self.entries.len() * core::mem::size_of::<u64>()
    }
}

impl Aml for XSDT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        for entry in &self.entries {
            sink.qword(*entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::XSDT;
    use crate::Aml;
    use alloc::vec::Vec;

    #[test]
    fn test_xsdt() {
        let mut bytes = Vec::new();
        let xsdt: &dyn Aml = &XSDT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);
        xsdt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_entry() {
        let mut xsdt = XSDT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);
        let mut last_len = 0;
        for i in 0..128 {
            let mut bytes = Vec::new();

            xsdt.add_entry((i * 42) as u64);

            xsdt.to_aml_bytes(&mut bytes);
            let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));

            let len = xsdt.len();
            assert!(len > last_len);
            last_len = len;
            assert_eq!(sum, 0);
        }
    }
}