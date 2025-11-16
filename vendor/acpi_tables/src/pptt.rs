// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use zerocopy::{byteorder, byteorder::LE, AsBytes};

use crate::{aml_as_bytes, u8sum, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;

pub struct PPTT {
    header: TableHeader,
    handle_offset: u32,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
}

impl PPTT {
    fn update_header(&mut self, sum: u8, len: u32) {
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

    pub fn add_processor(&mut self, node: ProcessorNode) -> ProcessorHandle {
        let old_offset = self.handle_offset;
        self.handle_offset += node.len() as u32;
        self.update_header(node.u8sum(), node.len() as u32);
        self.structures.push(Box::new(node));
        ProcessorHandle(old_offset)
    }

    pub fn add_cache(&mut self, node: CacheNode) -> CacheHandle {
        let old_offset = self.handle_offset;
        self.handle_offset += CacheNode::len() as u32;
        self.update_header(node.u8sum(), CacheNode::len() as u32);
        self.structures.push(Box::new(node));
        CacheHandle(old_offset)
    }

    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let header = TableHeader {
            signature: *b"PPTT",
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

        Self {
            header,
            handle_offset: TableHeader::len() as u32,
            checksum: cksum,
            structures: Vec::new(),
        }
    }
}

impl Aml for PPTT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ProcessorHandle(u32);
#[derive(Copy, Clone, Debug)]
pub struct CacheHandle(u32);

#[repr(u8)]
enum NodeType {
    Processor = 0,
    Cache = 1,
}

#[derive(Debug)]
pub struct ProcessorNode {
    pub flags: u32,
    pub parent: u32,
    pub acpi_processor_id: u32,
    resources: Vec<CacheHandle>,
}

impl ProcessorNode {
    const PHYSICAL: u32 = 1 << 0;
    const VALID: u32 = 1 << 1;
    const THREAD: u32 = 1 << 2;
    const LEAF: u32 = 1 << 3;
    const IDENTICAL: u32 = 1 << 4;

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len(&self) -> usize {
        20 + self.resources.len() * core::mem::size_of::<u32>()
    }

    pub fn new(parent: Option<&ProcessorHandle>, acpi_processor_id: u32) -> Self {
        Self {
            flags: 0,
            parent: match parent {
                None => 0,
                Some(handle) => handle.0,
            },
            acpi_processor_id,
            resources: Vec::new(),
        }
    }

    pub fn add_cache(mut self, c: &CacheHandle) -> Self {
        self.resources.push(*c);
        self
    }

    pub fn physical(mut self) -> Self {
        self.flags |= Self::PHYSICAL;
        self
    }

    pub fn valid(mut self) -> Self {
        self.flags |= Self::VALID;
        self
    }

    pub fn thread(mut self) -> Self {
        self.flags |= Self::THREAD;
        self
    }

    pub fn leaf(mut self) -> Self {
        self.flags |= Self::LEAF;
        self
    }

    pub fn identical(mut self) -> Self {
        self.flags |= Self::IDENTICAL;
        self
    }
}

impl Aml for ProcessorNode {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        let reserved: u16 = 0;

        sink.byte(NodeType::Processor as u8);
        sink.byte(self.len() as u8);
        sink.word(reserved);
        sink.dword(self.flags);
        sink.dword(self.parent);
        sink.dword(self.acpi_processor_id);
        sink.dword(self.resources.len() as u32);
        for r in &self.resources {
            sink.dword(r.0);
        }
    }
}

#[derive(Default)]
pub struct CacheNodeBuilder {
    next_level: u32,
    size: u32,
    set_count: u32,
    associativity: u8,
    attributes: u8,
    line_size: u16,
    id: u32,
    flags: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum AllocationType {
    Read = 0,
    Write = 1 << 0,
    Both = 1 << 1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CacheType {
    Data = 0 << 2,
    Instruction = 1 << 2,
    Unified = 1 << 3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum WritePolicy {
    Writeback = 0 << 4,
    Writethrough = 1 << 4,
}

impl CacheNodeBuilder {
    const SIZE: u32 = 1 << 0;
    const SETS: u32 = 1 << 1;
    const ASSOCIATIVITY: u32 = 1 << 2;
    const ALLOCATION: u32 = 1 << 3;
    const TYPE: u32 = 1 << 4;
    const POLICY: u32 = 1 << 5;
    const LINE_SIZE: u32 = 1 << 6;
    const ID: u32 = 1 << 7;

    pub fn next_level(mut self, c: &CacheHandle) -> Self {
        self.next_level = c.0;
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self.flags |= Self::SIZE;
        self
    }

    pub fn sets(mut self, set_count: u32) -> Self {
        self.set_count = set_count;
        self.flags |= Self::SETS;
        self
    }

    pub fn associativity(mut self, associativity: u8) -> Self {
        self.associativity = associativity;
        self.flags |= Self::ASSOCIATIVITY;
        self
    }

    pub fn allocation_type(mut self, a: AllocationType) -> Self {
        self.attributes |= a as u8;
        self.flags |= Self::ALLOCATION;
        self
    }

    pub fn cache_type(mut self, c: CacheType) -> Self {
        self.attributes |= c as u8;
        self.flags |= Self::TYPE;
        self
    }

    pub fn write_policy(mut self, w: WritePolicy) -> Self {
        self.attributes |= w as u8;
        self.flags |= Self::POLICY;
        self
    }

    pub fn line_size(mut self, line_size: u16) -> Self {
        self.line_size = line_size;
        self.flags |= Self::LINE_SIZE;
        self
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self.flags |= Self::ID;
        self
    }

    pub fn to_node(self) -> CacheNode {
        CacheNode {
            r#type: NodeType::Cache as u8,
            length: CacheNode::len() as u8,
            _reserved: 0.into(),
            flags: self.flags.into(),
            next_level: self.next_level.into(),
            size: self.size.into(),
            set_count: self.set_count.into(),
            associativity: self.associativity,
            attributes: self.attributes,
            line_size: self.line_size.into(),
            id: self.id.into(),
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, AsBytes)]
pub struct CacheNode {
    r#type: u8,
    length: u8,
    _reserved: U16,
    flags: U32,
    next_level: U32,
    size: U32,
    set_count: U32,
    associativity: u8,
    attributes: u8,
    line_size: U16,
    id: U32,
}

impl CacheNode {
    pub fn len() -> usize {
        28
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

aml_as_bytes!(CacheNode);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pptt() {
        let mut pptt = PPTT::new([0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0], 0);

        let llc = CacheNodeBuilder::default()
            .id(0x1000)
            .sets(16)
            .size(1024 * 1024)
            .associativity(2)
            .allocation_type(AllocationType::Both)
            .cache_type(CacheType::Unified)
            .write_policy(WritePolicy::Writeback)
            .line_size(64)
            .to_node();
        let llch = pptt.add_cache(llc);

        let mlc = CacheNodeBuilder::default()
            .id(0x1000)
            .sets(8)
            .size(64 * 1024)
            .associativity(4)
            .allocation_type(AllocationType::Read)
            .cache_type(CacheType::Instruction)
            .write_policy(WritePolicy::Writethrough)
            .line_size(64)
            .next_level(&llch)
            .to_node();
        let mlch = pptt.add_cache(mlc);

        let mut size = TableHeader::len() + CacheNode::len() * 2;
        for i in 0..128 {
            let cpu = ProcessorNode::new(None, i as u32)
                .physical()
                .valid()
                .leaf()
                .identical()
                .add_cache(&llch)
                .add_cache(&mlch);
            size += cpu.len();

            pptt.add_processor(cpu);
        }

        let mut bytes = Vec::new();
        pptt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(size, bytes.len());
    }
}