// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

extern crate alloc;

use crate::{u8sum, Aml, AmlSink, Checksum, TableHeader};
use alloc::{boxed::Box, vec, vec::Vec};
use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(C, packed)]
#[derive(Clone, Copy, Default, AsBytes)]
struct Header {
    table_header: TableHeader,
    _reserved: u32,
    timebase_frequency: U64,
    rhct_nodes: U32,
    array_offset: U32,
}

impl Header {
    fn len() -> usize {
        core::mem::size_of::<Header>()
    }
}

pub struct RHCT {
    header: Header,
    handle_offset: u32,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
}

#[derive(Debug)]
pub struct IsaStringHandle(u32);

#[derive(Debug)]
pub struct CmoHandle(u32);

impl RHCT {
    pub fn new(
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        timebase_frequency: u64,
    ) -> Self {
        let mut header = Header {
            table_header: TableHeader {
                signature: *b"RHCT",
                length: (Header::len() as u32).into(),
                revision: 1,
                checksum: 0,
                oem_id,
                oem_table_id,
                oem_revision: oem_revision.into(),
                creator_id: crate::CREATOR_ID,
                creator_revision: crate::CREATOR_REVISION,
            },
            _reserved: 0u32,
            timebase_frequency: timebase_frequency.into(),
            rhct_nodes: 0.into(),
            array_offset: 56.into(),
        };

        let mut cksum = Checksum::default();
        cksum.append(header.as_bytes());
        header.table_header.checksum = cksum.value();

        Self {
            header,
            handle_offset: Header::len() as u32,
            checksum: cksum,
            structures: Vec::new(),
        }
    }

    fn update_header(&mut self, sum: u8, len: u32) {
        let old_len = self.header.table_header.length.get();
        let new_len = len + old_len;
        self.header.table_header.length.set(new_len);

        // Update the current node count
        let old_node_count = self.header.rhct_nodes.get();
        let new_node_count = old_node_count + 1;
        self.header.rhct_nodes = new_node_count.into();
        self.checksum.delete(old_node_count.as_bytes());
        self.checksum.append(new_node_count.as_bytes());

        // Update the length and data
        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.add(sum);
        self.header.table_header.checksum = self.checksum.value();
    }

    pub fn add_isa_string(&mut self, string: &'static str) -> IsaStringHandle {
        let node = IsaStringNode { string };
        let old_offset = self.handle_offset;

        self.handle_offset += node.len() as u32;
        self.update_header(node.u8sum(), node.len() as u32);
        self.structures.push(Box::new(node));

        IsaStringHandle(old_offset)
    }

    pub fn add_mmu_node(&mut self, scheme: VirtualAddressScheme) {
        let node = MmuNode::new(scheme);

        self.handle_offset += MmuNode::len() as u32;
        self.update_header(node.u8sum(), MmuNode::len() as u32);
        self.structures.push(Box::new(node));
    }

    pub fn add_cmo(&mut self, cmo: CmoNode) -> CmoHandle {
        let old_offset = self.handle_offset;

        self.handle_offset += CmoNode::len() as u32;
        self.update_header(cmo.u8sum(), CmoNode::len() as u32);
        self.structures.push(Box::new(cmo));

        CmoHandle(old_offset)
    }

    pub fn add_hart_info(&mut self, hi: HartInfoNode) {
        self.handle_offset += hi.len() as u32;
        self.update_header(hi.u8sum(), hi.len() as u32);
        self.structures.push(Box::new(hi));
    }
}

impl Aml for RHCT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy)]
enum RhctNodeType {
    IsaString = 0,
    Cmo = 1,
    Mmu = 2,
    HartInfo = 65535,
}

pub struct IsaStringNode {
    string: &'static str,
}

impl IsaStringNode {
    const REVISION: u16 = 1;

    pub fn new(string: &'static str) -> Self {
        Self { string }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len(&self) -> usize {
        // type (2), length (2), revision (2), string length (2),
        // string (N), null terminator (1), padding (0 or 1)
        let len = 8 + self.string.len() + 1;
        if len % 2 == 0 {
            len
        } else {
            len + 1
        }
    }
}

impl Aml for IsaStringNode {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // ISA string length (including NULL terminator)
        let strlen = self.string.len() as u16 + 1;
        let padding_reqd = strlen % 2 == 1;
        sink.word(RhctNodeType::IsaString as u16);
        sink.word(self.len() as u16);
        sink.word(Self::REVISION);
        sink.word(strlen);
        for byte in self.string.bytes() {
            sink.byte(byte);
        }
        sink.byte(0); // NULL terminator
        if padding_reqd {
            sink.byte(0);
        }
    }
}

// Each entry in the array contains the address offset of a RHCT node
// relative to the start of the RHCT (e.g. the first element in the
// array can be the offset between the start of the RHCT table and the
// start of the appropriate ISA string node structure for this hart.)
// Each hart shall have at least one element in this array which
// points to an ISA node. If all harts have the same ISA string, then
// it is legal to have one IsaNodeString structure, and one
// HartInfoNode structure, which contains N offsets (one for each
// hart), and they all point to the same (single) IsaNodeString node.
pub struct HartInfoNode {
    processor_uid: u32,
    handles: Vec<u32>,
}

impl HartInfoNode {
    const REVISION: u16 = 1;

    pub fn new(processor_uid: u32, handle: &IsaStringHandle) -> Self {
        Self {
            processor_uid,
            handles: vec![handle.0],
        }
    }

    pub fn with_cmo(mut self, cmo: &CmoHandle) -> Self {
        self.handles.push(cmo.0);
        self
    }

    fn len(&self) -> usize {
        12 + 4 * self.handles.len()
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for HartInfoNode {
    // NOTE: assumes 1 handle for now
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let ty = RhctNodeType::HartInfo as u16;
        sink.word(ty);
        sink.word(self.len() as u16);
        sink.word(Self::REVISION);
        sink.word(self.handles.len() as u16);
        sink.dword(self.processor_uid);
        for handle in &self.handles {
            sink.dword(*handle);
        }
    }
}

pub struct CmoNode {
    cbom_block_size: u8,
    cbop_block_size: u8,
    cboz_block_size: u8,
}

impl CmoNode {
    pub fn new(
        cbom_block_size_pow2: u8,
        cbop_block_size_pow2: u8,
        cboz_block_size_pow2: u8,
    ) -> Self {
        Self {
            cbom_block_size: cbom_block_size_pow2,
            cbop_block_size: cbop_block_size_pow2,
            cboz_block_size: cboz_block_size_pow2,
        }
    }

    fn len() -> usize {
        10
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for CmoNode {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let ty = RhctNodeType::Cmo as u16;
        sink.word(ty);
        sink.word(Self::len() as u16);
        sink.word(1); // revision
        sink.byte(0); // reserved
        sink.byte(self.cbom_block_size);
        sink.byte(self.cbop_block_size);
        sink.byte(self.cboz_block_size);
    }
}

#[repr(u8)]
pub enum VirtualAddressScheme {
    Sv39 = 0,
    Sv48 = 1,
    Sv57 = 2,
}

pub struct MmuNode {
    supported_type: u8,
}

impl MmuNode {
    pub fn new(scheme: VirtualAddressScheme) -> Self {
        Self {
            supported_type: scheme as u8,
        }
    }

    fn len() -> usize {
        8
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for MmuNode {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let ty = RhctNodeType::Mmu as u16;
        sink.word(ty);
        sink.word(Self::len() as u16);
        sink.word(1); // revision
        sink.byte(0); // reserved
        sink.byte(self.supported_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhct() {
        let mut bytes = Vec::new();
        let rhct = RHCT::new(
            [b'R', b'I', b'V', b'O', b'S', 0],       /* oem_id */
            [b'R', b'I', b'V', b'O', b'S', 0, 0, 0], /* oem_table_id */
            42u32,                                   /* oem_revision */
            0x9012_1234_5678,                        /* timebase_frequency */
        );

        rhct.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_isa() {
        let mut bytes = Vec::new();
        let mut rhct = RHCT::new(
            [b'R', b'I', b'V', b'O', b'S', 0],       /* oem_id */
            [b'R', b'I', b'V', b'O', b'S', 0, 0, 0], /* oem_table_id */
            42u32,                                   /* oem_revision */
            0x9012_1234_5678,                        /* timebase_frequency */
        );

        let _ = rhct.add_isa_string("foobar");
        let _ = rhct.add_isa_string("blahblah");
        let _ = rhct.add_isa_string("quux");

        rhct.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_mmu() {
        let mut bytes = Vec::new();
        let mut rhct = RHCT::new(*b"RIVOS_", *b"RIVOS___", 42u32, 0x1111_2222_3333_4444);

        rhct.add_mmu_node(VirtualAddressScheme::Sv57);

        rhct.to_aml_bytes(&mut bytes);
        assert_eq!(
            bytes[core::mem::size_of::<Header>() + MmuNode::len() - 1],
            VirtualAddressScheme::Sv57 as u8
        );
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_hartinfo() {
        let mut bytes = Vec::new();
        let mut rhct = RHCT::new(
            [b'A', b'C', b'P', b'I', 0, 0],       /* oem_id */
            [b'A', b'C', b'P', b'I', 0, 0, 0, 0], /* oem_table_id */
            42u32,                                /* oem_revision */
            0x9012_1234_5678,                     /* timebase_frequency */
        );

        let h = rhct.add_isa_string("foobar");
        let b = rhct.add_isa_string("blah");

        for i in 0..128 {
            let hi = if i < 64 {
                HartInfoNode::new(i as u32, &h)
            } else {
                HartInfoNode::new(i as u32, &b)
            };

            rhct.add_hart_info(hi);
        }

        rhct.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}