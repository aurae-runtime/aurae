// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::AsBytes;

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{u8sum, Aml, AmlSink, Checksum, TableHeader};

pub struct VIOT {
    header: TableHeader,
    checksum: Checksum,
    handle_offset: u16,
    nodes: Vec<Box<dyn Aml>>,
}

const NODE_OFFSET: u16 = 48;

impl VIOT {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"VIOT",
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
        cksum.append(NODE_OFFSET.as_bytes());
        header.checksum = cksum.value();

        Self {
            header,
            checksum: cksum,
            handle_offset: Self::header_len() as u16,
            nodes: Vec::new(),
        }
    }

    fn header_len() -> usize {
        TableHeader::len() + 12
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

        // The header also contains a count of the number of nodes, so the
        // sum needs an additional '1' added to it.
        self.checksum.add(1);

        self.header.checksum = self.checksum.value();
    }

    pub fn add_pci_range(&mut self, range: PciRange) {
        self.update_header(range.u8sum(), PciRange::len() as u32);
        self.handle_offset += PciRange::len() as u16;
        self.nodes.push(Box::new(range));
    }

    pub fn add_mmio_endpoint(&mut self, ep: MmioEndpoint) {
        self.update_header(ep.u8sum(), MmioEndpoint::len() as u32);
        self.handle_offset += MmioEndpoint::len() as u16;
        self.nodes.push(Box::new(ep));
    }

    pub fn add_virtio_pci_iommu(&mut self, iommu: VirtIoPciIommu) -> TranslationHandle {
        let old_offset = self.handle_offset;
        self.update_header(iommu.u8sum(), VirtIoPciIommu::len() as u32);
        self.handle_offset += VirtIoPciIommu::len() as u16;
        self.nodes.push(Box::new(iommu));
        TranslationHandle(old_offset)
    }

    pub fn add_virtio_mmio_iommu(&mut self, iommu: VirtIoMmioIommu) -> TranslationHandle {
        let old_offset = self.handle_offset;
        self.update_header(iommu.u8sum(), VirtIoMmioIommu::len() as u32);
        self.handle_offset += VirtIoMmioIommu::len() as u16;
        self.nodes.push(Box::new(iommu));
        TranslationHandle(old_offset)
    }
}

/// The VIOT table describes the topology of paravirtualized I/O
/// translation devices and the endpoints they manage. It is intended
/// to replace the vendor-specific tables that have been used in the
/// past (e.g. IORT for ARM, DMAR for Intel, IVRS for AMD, etc.).
impl Aml for VIOT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.vec(self.header.as_bytes());
        sink.word(self.nodes.len() as u16);
        sink.word(NODE_OFFSET);
        sink.qword(0); // reserved

        for st in &self.nodes {
            st.to_aml_bytes(sink);
        }
    }
}

#[repr(u8)]
enum ViotEntryType {
    PciRange = 1,
    MmioEndpoint = 2,
    VirtIoPciIommu = 3,
    VirtIoMmioIommu = 4,
}

/// A handle returned for a translation-type device (IOMMU)
pub struct TranslationHandle(u16);

pub struct PciDevice {
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciDevice {
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        assert!(device < 32);
        assert!(function < 8);

        Self {
            segment,
            bus,
            device,
            function,
        }
    }

    fn as_bdf(&self) -> u16 {
        (self.bus as u16) << 8 | (self.device as u16) << 3 | self.function as u16
    }
}

/// This structure describes a range of PCI endpoints
pub struct PciRange {
    first: PciDevice,
    last: PciDevice,
    translation_offset: u16,
}

impl PciRange {
    pub fn new(first: PciDevice, last: PciDevice, translation_handle: &TranslationHandle) -> Self {
        Self {
            first,
            last,
            translation_offset: translation_handle.0,
        }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len() -> usize {
        24
    }
}

impl Aml for PciRange {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(ViotEntryType::PciRange as u8);
        sink.byte(0); // reserved
        sink.word(Self::len() as u16);
        sink.dword(self.first.as_bdf() as u32);
        sink.word(self.first.segment);
        sink.word(self.last.segment);
        sink.word(self.first.as_bdf());
        sink.word(self.last.as_bdf());
        sink.word(self.translation_offset);
        // 6 reserved bytes at the end
        sink.word(0);
        sink.dword(0);
    }
}

/// A single endpoint identified by its base MMIO address.
pub struct MmioEndpoint {
    endpoint: u32,
    base_addr: u64,
    translation_offset: u16,
}

impl MmioEndpoint {
    pub fn new(endpoint_id: u32, base_addr: u64, translation_handle: &TranslationHandle) -> Self {
        Self {
            endpoint: endpoint_id,
            base_addr,
            translation_offset: translation_handle.0,
        }
    }

    fn len() -> usize {
        24
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for MmioEndpoint {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(ViotEntryType::MmioEndpoint as u8);
        sink.byte(0); // reserved
        sink.word(Self::len() as u16);
        sink.dword(self.endpoint);
        sink.qword(self.base_addr);
        sink.word(self.translation_offset);
        // 6 reserved bytes
        sink.word(0);
        sink.dword(0);
    }
}

/// A virtio-iommu device (possibly based on virtio-pci transport)
pub struct VirtIoPciIommu {
    device: PciDevice,
}

impl VirtIoPciIommu {
    pub fn new(device: PciDevice) -> Self {
        Self { device }
    }

    fn len() -> usize {
        16
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for VirtIoPciIommu {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(ViotEntryType::VirtIoPciIommu as u8);
        sink.byte(0); // reserved
        sink.word(Self::len() as u16);
        sink.word(self.device.segment);
        sink.word(self.device.as_bdf());
        sink.qword(0); // reserved
    }
}

/// A virtio-iommu device based on virtio-mmio transport
pub struct VirtIoMmioIommu {
    base_addr: u64,
}

impl VirtIoMmioIommu {
    pub fn new(base_addr: u64) -> Self {
        Self { base_addr }
    }

    fn len() -> usize {
        16
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }
}

impl Aml for VirtIoMmioIommu {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(ViotEntryType::VirtIoMmioIommu as u8);
        sink.byte(0); // reserved
        sink.word(Self::len() as u16);
        sink.dword(0); // reserved
        sink.qword(self.base_addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viot() {
        let viot = VIOT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);

        let mut bytes = Vec::new();
        viot.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_viot_pci() {
        let mut viot = VIOT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);

        let handle = viot.add_virtio_pci_iommu(VirtIoPciIommu::new(PciDevice::new(5, 6, 7, 7)));

        viot.add_pci_range(PciRange::new(
            PciDevice::new(0, 0, 0, 0),
            PciDevice::new(1, 2, 3, 4),
            &handle,
        ));

        let len = VIOT::header_len() + VirtIoPciIommu::len() + PciRange::len();

        let mut bytes = Vec::new();
        viot.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), len);
    }

    #[test]
    fn test_viot_mmio() {
        let mut viot = VIOT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);

        let handle = viot.add_virtio_pci_iommu(VirtIoPciIommu::new(PciDevice::new(5, 6, 7, 7)));

        viot.add_mmio_endpoint(MmioEndpoint::new(
            0x1234_5678,
            0x0123_4567_8901_2345,
            &handle,
        ));

        let len = VIOT::header_len() + VirtIoPciIommu::len() + MmioEndpoint::len();

        let mut bytes = Vec::new();
        viot.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), len);
    }

    #[test]
    fn test_viot_mmio_iommu() {
        let mut viot = VIOT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef);

        let handle = viot.add_virtio_mmio_iommu(VirtIoMmioIommu::new(0x1234_5678_9012_3456));

        viot.add_mmio_endpoint(MmioEndpoint::new(
            0x1234_5678,
            0x0123_4567_8901_2345,
            &handle,
        ));

        let len = VIOT::header_len() + VirtIoMmioIommu::len() + MmioEndpoint::len();

        let mut bytes = Vec::new();
        viot.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), len);
    }
}