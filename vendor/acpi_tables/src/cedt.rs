// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::AsBytes;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{u8sum, Aml, AmlSink, Checksum, TableHeader};

pub struct CEDT {
    header: TableHeader,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
}

impl CEDT {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"CEDT",
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
            structures: Vec::new(),
        }
    }

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

    pub fn add_host_bridge(&mut self, chbs: CxlHostBridge) {
        self.update_header(chbs.u8sum(), CxlHostBridge::len() as u32);
        self.structures.push(Box::new(chbs));
    }

    pub fn add_fixed_memory(&mut self, cfmws: CxlFixedMemory) {
        self.update_header(cfmws.u8sum(), cfmws.len() as u32);
        self.structures.push(Box::new(cfmws));
    }

    pub fn add_xor_interleave_math(&mut self, xorm: XorInterleaveMath) {
        self.update_header(xorm.u8sum(), xorm.len() as u32);
        self.structures.push(Box::new(xorm));
    }

    pub fn add_port_association(&mut self, pa: PortAssociation) {
        self.update_header(pa.u8sum(), PortAssociation::len() as u32);
        self.structures.push(Box::new(pa));
    }
}

impl Aml for CEDT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        self.header.to_aml_bytes(sink);

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

#[repr(u8)]
enum CedtStructureType {
    Chbs = 0,
    Cfmws = 1,
    Cxims = 2,
    Rdpas = 3,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CxlVersion {
    Cxl1_1 = 0,
    Cxl2 = 1,
}

impl CxlVersion {
    fn len(&self) -> usize {
        match self {
            CxlVersion::Cxl1_1 => 0x2000,
            CxlVersion::Cxl2 => 0x1_0000,
        }
    }
}

/// CHBS - CXL Host Bridge Structure
pub struct CxlHostBridge {
    host_bridge_uid: u32,
    cxl_version: CxlVersion,
    port_base: u64,
}

impl CxlHostBridge {
    pub fn new(host_bridge_uid: u32, cxl_version: CxlVersion, port_base: u64) -> Self {
        Self {
            host_bridge_uid,
            cxl_version,
            port_base,
        }
    }

    fn len() -> usize {
        32
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for CxlHostBridge {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(CedtStructureType::Chbs as u8);
        sink.byte(0); // reserved
        sink.byte(Self::len() as u8);
        sink.dword(self.host_bridge_uid);
        sink.dword(self.cxl_version as u32);
        sink.qword(self.port_base);
        sink.qword(self.cxl_version.len() as u64);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum InterleaveArithmetic {
    Modulo = 0,
    ModuloXor = 1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum InterleaveGranularity {
    Granularity256b = 0,
    Granularity512b = 1,
    Granularity1kb = 2,
    Granularity2kb = 3,
    Granularity4kb = 4,
    Granularity8kb = 5,
    Granularity16kb = 6,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum InterleaveWays {
    Ways1 = 0,
    Ways2 = 1,
    Ways4 = 2,
    Ways8 = 3,
    /// The rest are valid for CXL.mem only (version 2+)
    Ways16 = 4,
    Ways3 = 8,
    Ways6 = 9,
    Ways12 = 10,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum WindowRestrictions {
    CxlType2Memory = 1 << 0,
    CxlType3Memory = 1 << 1,
    Volatile = 1 << 2,
    Persistent = 1 << 3,
    FixedConfiguration = 1 << 4,
}

/// CXL Fixed Memory Window Structure (CFMWS)
pub struct CxlFixedMemory {
    /// Must be 256 MiB-aligned
    base_addr: u64,
    /// Number of Interleaved Ways * 256 MiB
    size: u64,
    /// Arithmetic used for mapping physical addresses to interleave targets
    interleave_arithmetic: InterleaveArithmetic,
    /// Number of consecutive bytes assigned to each target
    interleave_granularity: InterleaveGranularity,
    /// Number of targets across which this memory range is interleaved
    interleave_ways: InterleaveWays,
    /// Restrictions placed on OSPM's use of this window.
    window_restrictions: u16,
    /// ID of the QoS Throttling Group associated with this window.
    qtg_id: u16,
    /// List of ACPI UIDs that are part of this fixed memory window;
    /// if an entry is a CXL Host Bridge, there must be a
    /// corresponding CHBS structure.
    interleave_targets: Vec<[u8; 4]>,
}

impl CxlFixedMemory {
    pub fn new(
        base_addr: u64,
        size: u64,
        arithmetic: InterleaveArithmetic,
        granularity: InterleaveGranularity,
        ways: InterleaveWays,
        qtg_id: u16,
    ) -> Self {
        Self {
            base_addr,
            size,
            interleave_arithmetic: arithmetic,
            interleave_granularity: granularity,
            interleave_ways: ways,
            window_restrictions: 0,
            qtg_id,
            interleave_targets: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        0x24 + 4 * self.num_interleaved_ways()
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    pub fn cxl_type_2_memory(mut self) -> Self {
        self.window_restrictions |= WindowRestrictions::CxlType2Memory as u16;
        self
    }

    pub fn cxl_type_3_memory(mut self) -> Self {
        self.window_restrictions |= WindowRestrictions::CxlType2Memory as u16;
        self
    }

    pub fn volatile(mut self) -> Self {
        self.window_restrictions |= WindowRestrictions::Volatile as u16;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.window_restrictions |= WindowRestrictions::Persistent as u16;
        self
    }

    pub fn fixed_configuration(mut self) -> Self {
        self.window_restrictions |= WindowRestrictions::FixedConfiguration as u16;
        self
    }

    pub fn add_target(&mut self, uid: [u8; 4]) {
        self.interleave_targets.push(uid);
    }

    fn num_interleaved_ways(&self) -> usize {
        match self.interleave_ways {
            InterleaveWays::Ways1 => 1,
            InterleaveWays::Ways2 => 2,
            InterleaveWays::Ways4 => 4,
            InterleaveWays::Ways8 => 8,
            InterleaveWays::Ways16 => 16,
            InterleaveWays::Ways3 => 3,
            InterleaveWays::Ways6 => 6,
            InterleaveWays::Ways12 => 12,
        }
    }
}

impl Aml for CxlFixedMemory {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Ensure the user added the same number of targets that they
        // specified in the constructor.
        assert_eq!(self.num_interleaved_ways(), self.interleave_targets.len());

        sink.byte(CedtStructureType::Cfmws as u8);
        sink.byte(0); // reserved
        sink.word(self.len() as u16);
        sink.dword(0); //reserved
        sink.qword(self.base_addr);
        sink.qword(self.size);
        sink.byte(self.interleave_ways as u8);
        sink.byte(self.interleave_arithmetic as u8);
        sink.word(0); // reserved
        sink.dword(self.interleave_granularity as u32);
        sink.word(self.window_restrictions);
        sink.word(self.qtg_id);
        for target in &self.interleave_targets {
            for byte in target {
                sink.byte(*byte);
            }
        }
    }
}

/// CXL XOR Interleave Math Structure
/// If a CFMWS entry reports Interleave Arithmetic of the `Modulo XOR`
/// type, then there must be on CXIMS entry associated with the HBIG
/// value in the CFMWS.
pub struct XorInterleaveMath {
    granularity: InterleaveGranularity,
    bitmaps: Vec<u64>,
}

impl XorInterleaveMath {
    pub fn new(granularity: InterleaveGranularity) -> Self {
        Self {
            granularity,
            bitmaps: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        8 + 8 * self.bitmaps.len()
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    pub fn add_xormap(&mut self, xormap: u64) {
        self.bitmaps.push(xormap);
    }
}

impl Aml for XorInterleaveMath {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(CedtStructureType::Cxims as u8);
        sink.byte(0); // reserved
        sink.word(self.len() as u16);
        sink.word(0); // reserved
        sink.byte(self.granularity as u8);
        sink.byte(self.bitmaps.len() as u8);
        for xormap in &self.bitmaps {
            sink.qword(*xormap);
        }
    }
}

/// RCEC Downstream Port Association Structure
/// Enables error handlers to locate the downstream port(s) that
/// report errors to a given RCEC.
pub struct PortAssociation {
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
    protocol: ProtocolType,
    base_addr: u64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ProtocolType {
    CxlIo = 0,
    CxlMem = 1,
}

impl PortAssociation {
    pub fn new(
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        protocol: ProtocolType,
        base_addr: u64,
    ) -> Self {
        assert!(device < 32);
        assert!(function < 8);

        Self {
            segment,
            bus,
            device,
            function,
            protocol,
            base_addr,
        }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len() -> usize {
        16
    }

    fn bdf(&self) -> u16 {
        (self.bus as u16) << 8 | (self.device as u16) << 3 | self.function as u16
    }
}

impl Aml for PortAssociation {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(CedtStructureType::Rdpas as u8);
        sink.byte(0); // reserved
        sink.word(Self::len() as u16);
        sink.word(self.segment);
        sink.word(self.bdf());
        sink.byte(self.protocol as u8);
        sink.qword(self.base_addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cedt() {
        let cedt = CEDT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
        );

        let mut bytes = Vec::new();
        cedt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_chbs() {
        let mut cedt = CEDT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
        );

        cedt.add_host_bridge(CxlHostBridge::new(0xcdef, CxlVersion::Cxl2, 0x8_1234_5678));

        let mut bytes = Vec::new();
        cedt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_cfmws() {
        let mut cedt = CEDT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
        );

        let mut fm = CxlFixedMemory::new(
            0x1020_3040_5060_7080,
            0x9000_1020_3040_5060,
            InterleaveArithmetic::Modulo,
            InterleaveGranularity::Granularity512b,
            InterleaveWays::Ways2,
            0xefef,
        )
        .volatile()
        .persistent()
        .cxl_type_2_memory()
        .cxl_type_3_memory()
        .fixed_configuration();

        fm.add_target(*b"CPU0");
        fm.add_target(*b"CPU1");
        cedt.add_fixed_memory(fm);

        let mut bytes = Vec::new();
        cedt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_cxims() {
        let mut cedt = CEDT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
        );

        let mut xorm = XorInterleaveMath::new(InterleaveGranularity::Granularity16kb);

        xorm.add_xormap(0x1004_1803_2489_1384);
        xorm.add_xormap(0x2489_1384_1004_1803);
        cedt.add_xor_interleave_math(xorm);

        let mut bytes = Vec::new();
        cedt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_rdpas() {
        let mut cedt = CEDT::new(
            [b'A', b'B', b'C', b'D', b'E', b'F'],
            [b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H'],
            0x1234_5678,
        );

        cedt.add_port_association(PortAssociation::new(
            0x100,
            0xfe,
            0x1f,
            0x7,
            ProtocolType::CxlMem,
            0x1234_5678_90ab_cdef,
        ));

        let mut bytes = Vec::new();
        cedt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}