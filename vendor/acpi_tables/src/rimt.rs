// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::AsBytes;

extern crate alloc;
use alloc::{boxed::Box, string::String, vec::Vec};

use crate::{u8sum, Aml, AmlSink, Checksum, TableHeader};

#[derive(Copy, Clone)]
pub struct IommuOffset(u32);

pub struct RIMT {
    header: TableHeader,
    checksum: Checksum,
    handle_offset: usize,
    devices: Vec<Box<dyn Aml>>,
}

impl RIMT {
    const DEVICE_OFFSET: u32 = 48;

    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"RIMT",
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
        cksum.append(Self::DEVICE_OFFSET.as_bytes());
        header.checksum = cksum.value();

        Self {
            header,
            checksum: cksum,
            handle_offset: Self::header_len(),
            devices: Vec::new(),
        }
    }

    fn header_len() -> usize {
        TableHeader::len() + 12
    }

    // Only expected to be invoked when adding a new device
    // to the list of devices associated with the table.
    fn update_header(&mut self, sum: u8, len: u32) {
        let old_len = self.header.length.get();
        let new_len = len + old_len;
        self.header.length.set(new_len);

        // Remove the bytes from the old length, add the new length
        // and the new data.
        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.add(sum);

        // The header also contains a count of the number of devices,
        // so the sum needs an additional '1' added to it.
        self.checksum.add(1);

        self.header.checksum = self.checksum.value();
    }

    pub fn add_iommu(&mut self, iommu: Iommu) -> IommuOffset {
        let iommu_offset = IommuOffset(self.handle_offset as u32);
        self.update_header(iommu.u8sum(), iommu.len() as u32);
        self.handle_offset += iommu.len();
        self.devices.push(Box::new(iommu));
        iommu_offset
    }

    pub fn add_pcie_root_complex(&mut self, pcie_rc: PcieRootComplex) {
        self.update_header(pcie_rc.u8sum(), pcie_rc.len() as u32);
        self.handle_offset += pcie_rc.len();
        self.devices.push(Box::new(pcie_rc));
    }

    pub fn add_platform(&mut self, platform: Platform) {
        self.update_header(platform.u8sum(), platform.len() as u32);
        self.handle_offset += platform.len();
        self.devices.push(Box::new(platform));
    }
}

/// The RISC-V IO Mapping Table (RIMT) provides information about
/// the RISC-V IOMMU, describing the relationship between the IO
/// topology and the IOMMU.
impl Aml for RIMT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        sink.dword(self.devices.len() as u32);
        sink.dword(Self::DEVICE_OFFSET);
        sink.dword(0); // reserved

        for dev in &self.devices {
            dev.to_aml_bytes(sink);
        }
    }
}

#[repr(u8)]
enum RimtDeviceType {
    Iommu = 0,
    PcieRootComplex = 1,
    Platform = 2,
}

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

    fn as_segment(&self) -> u16 {
        self.segment
    }
}

pub struct InterruptWire {
    num: u32,
    level_trig: bool,
    polarity_high: bool,
    aplic_id: u16,
}

impl InterruptWire {
    pub fn new(num: u32, level_trig: bool, polarity_high: bool, aplic_id: u16) -> Self {
        Self {
            num,
            level_trig,
            polarity_high,
            aplic_id,
        }
    }

    fn len() -> usize {
        8
    }

    fn flags(&self) -> u16 {
        let mut flags = 0;
        if self.level_trig {
            flags |= 0x1;
        }
        if self.polarity_high {
            flags |= 0x2;
        }
        flags
    }
}

impl Aml for InterruptWire {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Interrupt number
        sink.dword(self.num);
        // Flags
        sink.word(self.flags());
        // APLIC ID
        sink.word(self.aplic_id);
    }
}

/// This structure describes an IOMMU device
pub struct Iommu {
    id: u16,
    base_addr: Option<u64>,
    pci_device: Option<PciDevice>,
    proximity_domain: Option<u32>,
    int_wires: Option<Vec<InterruptWire>>,
}

impl Iommu {
    const INTERRUPT_WIRE_OFFSET: u16 = 32;

    pub fn new(
        id: u16,
        base_addr: Option<u64>,
        pci_device: Option<PciDevice>,
        proximity_domain: Option<u32>,
        int_wires: Option<Vec<InterruptWire>>,
    ) -> Self {
        Self {
            id,
            base_addr,
            pci_device,
            proximity_domain,
            int_wires,
        }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len(&self) -> usize {
        Self::INTERRUPT_WIRE_OFFSET as usize + (InterruptWire::len() * self.num_int_wires())
    }

    fn flags(&self) -> u32 {
        let mut flags = 0;
        if self.pci_device.is_some() {
            flags |= 0x1;
        }
        if self.proximity_domain.is_some() {
            flags |= 0x2;
        }
        flags
    }

    fn num_int_wires(&self) -> usize {
        self.int_wires.as_ref().map_or(0, |i| i.len())
    }
}

impl Aml for Iommu {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Type
        sink.byte(RimtDeviceType::Iommu as u8);
        // Revision
        sink.byte(1);
        // Length
        sink.word(self.len() as u16);
        // ID
        sink.word(self.id);
        // Model
        sink.word(0);
        // Base Address
        sink.qword(self.base_addr.unwrap_or(0));
        // Flags
        sink.dword(self.flags());
        // PCIe Segment number
        sink.word(self.pci_device.as_ref().map_or(0, |d| d.as_segment()));
        // PCIe B/D/F
        sink.word(self.pci_device.as_ref().map_or(0, |d| d.as_bdf()));
        // Proximity Domain
        sink.dword(self.proximity_domain.unwrap_or(0));
        // Number of Interrupt Wires
        sink.word(self.num_int_wires() as u16);
        // Interrupt Wire Array Offset
        sink.word(Self::INTERRUPT_WIRE_OFFSET);
        // Interrupt Wire Array
        if let Some(int_wires) = self.int_wires.as_ref() {
            for int_wire in int_wires {
                int_wire.to_aml_bytes(sink);
            }
        }
    }
}

pub struct IdMapping {
    src_id: u32,
    dst_id: u32,
    num_ids: u32,
    dst_iommu_offset: IommuOffset,
    ats: bool,
    pri: bool,
    rciep: bool,
}

impl IdMapping {
    pub fn new(
        src_id: u32,
        dst_id: u32,
        num_ids: u32,
        dst_iommu_offset: IommuOffset,
        ats: bool,
        pri: bool,
        rciep: bool,
    ) -> Self {
        Self {
            src_id,
            dst_id,
            num_ids,
            dst_iommu_offset,
            ats,
            pri,
            rciep,
        }
    }

    fn len() -> usize {
        20
    }

    fn flags(&self) -> u32 {
        let mut flags = 0;
        if self.ats {
            flags |= 0x1;
        }
        if self.pri {
            flags |= 0x2;
        }
        if self.rciep {
            flags |= 0x4;
        }
        flags
    }
}

impl Aml for IdMapping {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Source ID Base
        sink.dword(self.src_id);
        // Destination ID Base
        sink.dword(self.dst_id);
        // Number of IDs
        sink.dword(self.num_ids);
        // Destination IOMMU offset
        sink.dword(self.dst_iommu_offset.0);
        // Flags
        sink.dword(self.flags());
    }
}

/// This structure describes a PCIe Root Complex device
pub struct PcieRootComplex {
    id: u16,
    pci_segment: u16,
    ats: bool,
    pri: bool,
    id_mappings: Option<Vec<IdMapping>>,
}

impl PcieRootComplex {
    const ID_MAPPING_OFFSET: u16 = 16;

    pub fn new(
        id: u16,
        pci_segment: u16,
        ats: bool,
        pri: bool,
        id_mappings: Option<Vec<IdMapping>>,
    ) -> Self {
        Self {
            id,
            pci_segment,
            ats,
            pri,
            id_mappings,
        }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn len(&self) -> usize {
        Self::ID_MAPPING_OFFSET as usize + (IdMapping::len() * self.num_id_mappings())
    }

    fn flags(&self) -> u32 {
        let mut flags = 0;
        if self.ats {
            flags |= 0x1;
        }
        if self.pri {
            flags |= 0x2;
        }
        flags
    }

    fn num_id_mappings(&self) -> usize {
        self.id_mappings.as_ref().map_or(0, |i| i.len())
    }
}

impl Aml for PcieRootComplex {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Type
        sink.byte(RimtDeviceType::PcieRootComplex as u8);
        // Revision
        sink.byte(1);
        // Length
        sink.word(self.len() as u16);
        // ID
        sink.word(self.id);
        // PCI Segment number
        sink.word(self.pci_segment);
        // Flags
        sink.dword(self.flags());
        // ID Mapping Array Offset
        sink.word(Self::ID_MAPPING_OFFSET);
        // Number of ID Mappings
        sink.word(self.num_id_mappings() as u16);
        // ID Mapping Array
        if let Some(id_mappings) = self.id_mappings.as_ref() {
            for id_mapping in id_mappings {
                id_mapping.to_aml_bytes(sink);
            }
        }
    }
}

/// This structure describes a platform device
pub struct Platform {
    id: u16,
    name: String,
    id_mappings: Option<Vec<IdMapping>>,
}

impl Platform {
    const NAME_OFFSET: usize = 12;

    pub fn new(id: u16, name: String, id_mappings: Option<Vec<IdMapping>>) -> Self {
        Self {
            id,
            name,
            id_mappings,
        }
    }

    fn u8sum(&self) -> u8 {
        u8sum(self)
    }

    fn id_mapping_offset(&self) -> usize {
        Self::NAME_OFFSET + self.name.as_bytes().len() + 1
    }

    fn len(&self) -> usize {
        self.id_mapping_offset() + (IdMapping::len() * self.num_id_mappings())
    }

    fn num_id_mappings(&self) -> usize {
        self.id_mappings.as_ref().map_or(0, |i| i.len())
    }
}

impl Aml for Platform {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        // Type
        sink.byte(RimtDeviceType::Platform as u8);
        // Revision
        sink.byte(1);
        // Length
        sink.word(self.len() as u16);
        // ID
        sink.word(self.id);
        // Reserved
        sink.word(0);
        // ID Mapping Array Offset
        sink.word(self.id_mapping_offset() as u16);
        // Number of ID Mappings
        sink.word(self.num_id_mappings() as u16);
        // Name
        for b in self.name.as_bytes() {
            sink.byte(*b);
        }
        // Null terminated ASCII string for the name
        sink.byte(0);
        // ID Mapping Array
        if let Some(id_mappings) = self.id_mappings.as_ref() {
            for id_mapping in id_mappings {
                id_mapping.to_aml_bytes(sink);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn rimt() -> RIMT {
        RIMT::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef)
    }

    fn interrupt_wires() -> Vec<InterruptWire> {
        vec![
            InterruptWire::new(1, true, false, 1),
            InterruptWire::new(2, false, true, 2),
            InterruptWire::new(3, true, true, 3),
            InterruptWire::new(4, false, false, 4),
        ]
    }

    fn id_mappings(iommu_offset: IommuOffset) -> Vec<IdMapping> {
        vec![
            IdMapping::new(1, 2, 10, iommu_offset, true, true, false),
            IdMapping::new(2, 3, 5, iommu_offset, false, false, true),
        ]
    }

    fn rimt_w_iommu(pci: bool) -> (RIMT, IommuOffset) {
        let mut rimt = rimt();

        let iommu_offset = if pci {
            rimt.add_iommu(Iommu::new(
                1,
                None,
                Some(PciDevice::new(5, 6, 7, 7)),
                Some(15),
                Some(interrupt_wires()),
            ))
        } else {
            rimt.add_iommu(Iommu::new(
                1,
                Some(0x1000),
                None,
                Some(15),
                Some(interrupt_wires()),
            ))
        };

        (rimt, iommu_offset)
    }

    #[test]
    fn test_rimt_header_length() {
        assert_eq!(RIMT::header_len(), 48);
    }

    #[test]
    fn test_rimt() {
        let rimt = rimt();
        let mut bytes = Vec::new();
        rimt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 48);
        assert_eq!(
            bytes,
            vec![
                82, 73, 77, 84, 48, 0, 0, 0, 1, 23, 70, 79, 79, 66, 65, 82, 67, 65, 70, 69, 68, 69,
                65, 68, 239, 190, 173, 222, 82, 86, 65, 84, 0, 0, 0, 1, 0, 0, 0, 0, 48, 0, 0, 0, 0,
                0, 0, 0
            ]
        );
    }

    #[test]
    fn test_rimt_pci() {
        let (mut rimt, iommu_offset) = rimt_w_iommu(true);

        rimt.add_pcie_root_complex(PcieRootComplex::new(
            1,
            1,
            true,
            true,
            Some(id_mappings(iommu_offset)),
        ));

        let mut bytes = Vec::new();
        rimt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 168);
        assert_eq!(
            bytes,
            vec![
                82, 73, 77, 84, 168, 0, 0, 0, 1, 242, 70, 79, 79, 66, 65, 82, 67, 65, 70, 69, 68,
                69, 65, 68, 239, 190, 173, 222, 82, 86, 65, 84, 0, 0, 0, 1, 2, 0, 0, 0, 48, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 64, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 5, 0,
                63, 6, 15, 0, 0, 0, 4, 0, 32, 0, 1, 0, 0, 0, 1, 0, 1, 0, 2, 0, 0, 0, 2, 0, 2, 0, 3,
                0, 0, 0, 3, 0, 3, 0, 4, 0, 0, 0, 0, 0, 4, 0, 1, 1, 56, 0, 1, 0, 1, 0, 3, 0, 0, 0,
                16, 0, 2, 0, 1, 0, 0, 0, 2, 0, 0, 0, 10, 0, 0, 0, 48, 0, 0, 0, 3, 0, 0, 0, 2, 0, 0,
                0, 3, 0, 0, 0, 5, 0, 0, 0, 48, 0, 0, 0, 4, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_rimt_platform() {
        let (mut rimt, iommu_offset) = rimt_w_iommu(false);

        rimt.add_platform(Platform::new(
            1,
            String::from("FULL.PATH.TO.DEVICE"),
            Some(id_mappings(iommu_offset)),
        ));

        let mut bytes = Vec::new();
        rimt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 184);
        assert_eq!(
            bytes,
            vec![
                82, 73, 77, 84, 184, 0, 0, 0, 1, 195, 70, 79, 79, 66, 65, 82, 67, 65, 70, 69, 68,
                69, 65, 68, 239, 190, 173, 222, 82, 86, 65, 84, 0, 0, 0, 1, 2, 0, 0, 0, 48, 0, 0,
                0, 0, 0, 0, 0, 0, 1, 64, 0, 1, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
                0, 0, 15, 0, 0, 0, 4, 0, 32, 0, 1, 0, 0, 0, 1, 0, 1, 0, 2, 0, 0, 0, 2, 0, 2, 0, 3,
                0, 0, 0, 3, 0, 3, 0, 4, 0, 0, 0, 0, 0, 4, 0, 2, 1, 72, 0, 1, 0, 0, 0, 32, 0, 2, 0,
                70, 85, 76, 76, 46, 80, 65, 84, 72, 46, 84, 79, 46, 68, 69, 86, 73, 67, 69, 0, 1,
                0, 0, 0, 2, 0, 0, 0, 10, 0, 0, 0, 48, 0, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0,
                5, 0, 0, 0, 48, 0, 0, 0, 4, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_interrupt_wire() {
        let int_wire = InterruptWire::new(1, false, false, 1);
        assert_eq!(int_wire.flags(), 0);
        let int_wire = InterruptWire::new(1, true, false, 1);
        assert_eq!(int_wire.flags(), 1);
        let int_wire = InterruptWire::new(1, false, true, 1);
        assert_eq!(int_wire.flags(), 2);
        let int_wire = InterruptWire::new(1, true, true, 1);
        assert_eq!(int_wire.flags(), 3);
        let mut bytes = Vec::new();
        int_wire.to_aml_bytes(&mut bytes);
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, vec![1, 0, 0, 0, 3, 0, 1, 0]);
    }

    #[test]
    fn test_iommu() {
        let iommu = Iommu::new(1, Some(0x1000), None, None, None);
        assert_eq!(iommu.flags(), 0);
        assert_eq!(iommu.num_int_wires(), 0);
        assert_eq!(iommu.len(), 32);
        let iommu = Iommu::new(
            1,
            None,
            Some(PciDevice::new(1, 2, 3, 4)),
            None,
            Some(interrupt_wires()),
        );
        assert_eq!(iommu.flags(), 1);
        assert_eq!(iommu.num_int_wires(), 4);
        assert_eq!(iommu.len(), 64);
        let iommu = Iommu::new(1, Some(0x1000), None, Some(15), None);
        assert_eq!(iommu.flags(), 2);
        let iommu = Iommu::new(1, None, Some(PciDevice::new(1, 2, 3, 4)), Some(15), None);
        assert_eq!(iommu.flags(), 3);
        let mut bytes = Vec::new();
        iommu.to_aml_bytes(&mut bytes);
        assert_eq!(bytes.len(), 32);
        assert_eq!(
            bytes,
            vec![
                0, 1, 32, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 28, 2, 15, 0, 0,
                0, 0, 0, 32, 0
            ]
        );
    }

    #[test]
    fn test_id_mapping() {
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), false, false, false);
        assert_eq!(id_mapping.flags(), 0);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), true, false, false);
        assert_eq!(id_mapping.flags(), 1);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), false, true, false);
        assert_eq!(id_mapping.flags(), 2);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), true, true, false);
        assert_eq!(id_mapping.flags(), 3);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), false, false, true);
        assert_eq!(id_mapping.flags(), 4);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), true, false, true);
        assert_eq!(id_mapping.flags(), 5);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), false, true, true);
        assert_eq!(id_mapping.flags(), 6);
        let id_mapping = IdMapping::new(1, 2, 3, IommuOffset(48), true, true, true);
        assert_eq!(id_mapping.flags(), 7);
        let mut bytes = Vec::new();
        id_mapping.to_aml_bytes(&mut bytes);
        assert_eq!(bytes.len(), 20);
        assert_eq!(
            bytes,
            vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 48, 0, 0, 0, 7, 0, 0, 0]
        );
    }

    #[test]
    fn test_pcie_root_complex() {
        let pcierc = PcieRootComplex::new(1, 1, false, false, None);
        assert_eq!(pcierc.flags(), 0);
        assert_eq!(pcierc.num_id_mappings(), 0);
        assert_eq!(pcierc.len(), 16);
        let pcierc = PcieRootComplex::new(1, 1, true, false, Some(id_mappings(IommuOffset(48))));
        assert_eq!(pcierc.flags(), 1);
        assert_eq!(pcierc.num_id_mappings(), 2);
        assert_eq!(pcierc.len(), 56);
        let pcierc = PcieRootComplex::new(1, 1, false, true, Some(id_mappings(IommuOffset(48))));
        assert_eq!(pcierc.flags(), 2);
        let pcierc = PcieRootComplex::new(1, 1, true, true, Some(id_mappings(IommuOffset(48))));
        assert_eq!(pcierc.flags(), 3);
        let mut bytes = Vec::new();
        pcierc.to_aml_bytes(&mut bytes);
        assert_eq!(bytes.len(), 56);
        assert_eq!(
            bytes,
            vec![
                1, 1, 56, 0, 1, 0, 1, 0, 3, 0, 0, 0, 16, 0, 2, 0, 1, 0, 0, 0, 2, 0, 0, 0, 10, 0, 0,
                0, 48, 0, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 5, 0, 0, 0, 48, 0, 0, 0, 4, 0,
                0, 0
            ]
        );
    }

    #[test]
    fn test_platform() {
        let platform = Platform::new(1, String::from("SHORT.NAME"), None);
        assert_eq!(platform.num_id_mappings(), 0);
        assert_eq!(platform.id_mapping_offset(), 23);
        assert_eq!(platform.len(), 23);
        let platform = Platform::new(
            1,
            String::from("MUCH.LONGER.NAME"),
            Some(id_mappings(IommuOffset(48))),
        );
        assert_eq!(platform.num_id_mappings(), 2);
        assert_eq!(platform.id_mapping_offset(), 29);
        assert_eq!(platform.len(), 69);
        let mut bytes = Vec::new();
        platform.to_aml_bytes(&mut bytes);
        assert_eq!(bytes.len(), 69);
        assert_eq!(
            bytes,
            vec![
                2, 1, 69, 0, 1, 0, 0, 0, 29, 0, 2, 0, 77, 85, 67, 72, 46, 76, 79, 78, 71, 69, 82,
                46, 78, 65, 77, 69, 0, 1, 0, 0, 0, 2, 0, 0, 0, 10, 0, 0, 0, 48, 0, 0, 0, 3, 0, 0,
                0, 2, 0, 0, 0, 3, 0, 0, 0, 5, 0, 0, 0, 48, 0, 0, 0, 4, 0, 0, 0
            ]
        );
    }
}