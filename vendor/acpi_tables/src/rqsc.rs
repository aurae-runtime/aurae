// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;
use alloc::vec::Vec;

use core::mem::size_of;

use crate::{aml_as_bytes, assert_same_size, gas, Aml, AmlSink, Checksum, TableHeader};

type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(u8)]
pub enum ControllerType {
    Capacity = 0,
    Bandwidth = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum ResourceType {
    Cache = 0,
    Memory = 1,
}

pub struct RQSC {
    header: TableHeader,
    structures: Vec<QoSController>,
}

impl RQSC {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut cksum = Checksum::default();

        let mut header = TableHeader {
            signature: *b"RQSC",
            length: (TableHeader::len() as u32).into(),
            revision: 1,
            checksum: 0,
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
        };
        cksum.append(header.as_bytes());
        header.checksum = cksum.value();

        Self {
            header,
            structures: Vec::new(),
        }
    }

    fn update_header(&mut self, qos_len: usize) {
        // Fix up the length of the table
        let len = qos_len as u32;
        let old_len = self.header.length.get();
        let new_len = len + old_len;
        self.header.length.set(new_len);

        // Fix up checksum
        self.header.checksum = 0;
        let mut cksum = Checksum::default();
        self.to_aml_bytes(&mut cksum);
        self.header.checksum = cksum.value();
    }

    pub fn add_controller(&mut self, q: QoSController) {
        let len = q.len();
        self.structures.push(q);
        self.update_header(len);
    }
}

impl Aml for RQSC {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        sink.dword(self.structures.len() as u32);
        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct QoSController {
    /// Identifies the specific register interface that is supported by this
    /// controller
    controller_type: u8,

    length: u16,
    /// Register Buffer describing the starting address of the QoS register interface
    register: gas::GAS,

    /// Non-zero number indicates that the controller supports allocation capability and the
    /// number of Resource Control IDs (RCID) supported by the controller. If 0, then no
    /// allocation control is available.
    rcid_count: u32,
    /// Non-zero number indicates that the controller supports usage monitoring capability and
    /// the number of Monitoring Control IDs (MCID) supported by the controller. If 0, then no
    /// usage monitoring is available.
    mcid_count: u32,

    /// Controller Specific flags.
    /// - Bit 0: When set, indicates the controller supports zero (0)
    ///   reservations/allocations.
    /// - Bit 1-7: Reserved
    /// - Bit 8-15: Vendor Specific
    controller_flags: u16,

    /// Number of Resource structures associated with this specific QoS
    /// controller.
    number_of_resources: u16,

    /// List of Resource Structures asssociated with this specific QoS
    /// controller.
    resource_structure: Vec<ResourceStructure>,
}

impl QoSController {
    pub fn new(
        controller_type: ControllerType,
        register_interface_address: gas::GAS,
        rcid_count: u32,
        mcid_count: u32,
        controller_flags: u16,
    ) -> Self {
        Self {
            controller_type: controller_type as u8,
            length: 28u16,
            register: register_interface_address,
            rcid_count,
            mcid_count,
            controller_flags,
            number_of_resources: 0,
            resource_structure: Vec::new(),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.length as usize
    }

    pub fn add_resource(&mut self, resource: ResourceStructure) {
        self.number_of_resources += 1;
        self.length += resource.len() as u16;
        self.resource_structure.push(resource);
    }
}

impl Aml for QoSController {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(self.controller_type);
        sink.byte(0);
        sink.word(self.length);
        self.register.to_aml_bytes(sink);
        sink.dword(self.rcid_count);
        sink.dword(self.mcid_count);
        sink.word(self.controller_flags);
        sink.word(self.number_of_resources);
        self.resource_structure
            .iter()
            .for_each(|resource| resource.to_aml_bytes(sink));
    }
}

/// Resource Structures asssociated with a specific QoS controller
#[derive(Clone, Debug)]
pub struct ResourceStructure {
    /// Describes the type of resource that this QoS controller has control over
    /// - 0: Cache
    /// - 1: Memory
    /// - 2-0x7F: Reserved
    /// - 0x80-0xFF: Vendor Specific
    resource_type: ResourceType,

    /// Length of this specific Resource Structure. Length includes the Resource
    /// Specific Data bytes as well. If length is set to 20, then, it indicates
    /// there is no resource specific data available for this structure.
    length: u16,

    /// Resource Type Specific flags
    /// - Bits 0-7: Reserved
    /// - Bits 8-15: Vendor Specific
    resource_flags: u16,

    /// [ResourceID]
    resource_id: ResourceID,
}

impl ResourceStructure {
    pub fn new(resource_type: ResourceType, resource_flags: u16, resource_id: ResourceID) -> Self {
        let length = size_of::<u8>() * 3 + size_of::<u16>() * 2 + resource_id.len();

        Self {
            resource_type,
            length: length as u16,
            resource_flags,
            resource_id,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.length as usize
    }
}

impl Aml for ResourceStructure {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(self.resource_type as u8);
        sink.byte(0);
        sink.word(self.length);
        sink.word(self.resource_flags);
        sink.byte(0);
        self.resource_id.to_aml_bytes(sink)
    }
}

/// Regroups Resource ID Type, Resource ID 1, Resource ID 2 and Resource
/// Specific Data.
/// Enforces coherency by using enum variants, ensuring a correct layout of Resource ID
/// 1 and 2 for a corresponding Resource ID Type.
#[derive(Clone, Debug)]
pub enum ResourceID {
    Cache(CacheResource),
    MemoryAffinityStructure(MemoryAffinityStructureResource),
    ACPIDevice(ACPIDeviceResource),
    PCIDevice(PCIDeviceResource),

    /// The byte must be the Resource ID Type.
    /// The Vec must be Resource ID 1, Resource ID 2 and Resource Specific Data
    /// serialized as aml_bytes.
    VendorSpecific(u8, Vec<u8>),
}

impl ResourceID {
    pub fn resource_id_type(&self) -> u8 {
        match self {
            ResourceID::Cache(_) => 0,
            ResourceID::MemoryAffinityStructure(_) => 1,
            ResourceID::ACPIDevice(_) => 2,
            ResourceID::PCIDevice(_) => 3,
            ResourceID::VendorSpecific(r#type, _) => *r#type,
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        size_of::<u8>()
            + match self {
                ResourceID::Cache(_) => size_of::<CacheResource>(),
                ResourceID::MemoryAffinityStructure(_) => {
                    size_of::<MemoryAffinityStructureResource>()
                }
                ResourceID::ACPIDevice(_) => size_of::<ACPIDeviceResource>(),
                ResourceID::PCIDevice(_) => size_of::<PCIDeviceResource>(),
                ResourceID::VendorSpecific(_, resource) => resource.len(),
            }
    }
}

impl Aml for ResourceID {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.byte(self.resource_id_type());

        match self {
            ResourceID::Cache(resource) => resource.to_aml_bytes(sink),
            ResourceID::MemoryAffinityStructure(resource) => resource.to_aml_bytes(sink),
            ResourceID::ACPIDevice(resource) => resource.to_aml_bytes(sink),
            ResourceID::PCIDevice(resource) => resource.to_aml_bytes(sink),
            ResourceID::VendorSpecific(_, resource) => sink.vec(resource.as_bytes()),
        }
    }
}

#[derive(Clone, Debug, Default, AsBytes)]
#[repr(C, packed)]
pub struct CacheResource {
    // Resource ID 1
    /// Unique Cache ID from the PPTT table’s Cache Type Structure (Table 5.159
    /// in ACPI Specification 6.5) that this controller is associated with.
    cache_id: U32,
    _reserved_resource_id_1: U32,

    // Resource ID 2
    _reserved_resource_id_2: U32,
}

impl CacheResource {
    pub fn new(cache_id: u32) -> Self {
        Self {
            cache_id: cache_id.into(),
            _reserved_resource_id_1: 0.into(),
            _reserved_resource_id_2: 0.into(),
        }
    }
}

#[derive(Clone, Debug, Default, AsBytes)]
#[repr(C, packed)]
pub struct MemoryAffinityStructureResource {
    // Resource ID 1
    /// Proximity domain from the SRAT table’s Memory Affinity Structure the
    /// resource is associated with. If the SRAT table is not implemented, then
    /// this value shall be 0 indicating a UMA memory configuration.
    proximity_domain: U32,
    _reserved_resource_id_1: U32,

    // Resource ID 2
    _reserved_resource_id_2: U32,

    // Resource Specific Data
    /// Indicates the actual raw bandwidth that each unit of bandwidth block
    /// corresponds to in bytes/seconds for this specific Resource.
    raw_bandwidth_per_block: U64,
}

impl MemoryAffinityStructureResource {
    pub fn new(proximity_domain: u32, raw_bandwidth_per_block: u64) -> Self {
        Self {
            proximity_domain: proximity_domain.into(),
            _reserved_resource_id_1: 0.into(),
            _reserved_resource_id_2: 0.into(),
            raw_bandwidth_per_block: raw_bandwidth_per_block.into(),
        }
    }
}

#[derive(Clone, Debug, Default, AsBytes)]
#[repr(C, packed)]
pub struct ACPIDeviceResource {
    // Resource ID 1
    /// _HID value of the ACPI Device corresponding to the Resource.
    acpi_hardware_id: U64,

    // Resource ID 2
    /// _UID value of the ACPI Device corresponding to the Resource.
    acpi_unique_id: U32,
}

impl ACPIDeviceResource {
    pub fn new(acpi_hardware_id: u64, acpi_unique_id: u32) -> Self {
        Self {
            acpi_hardware_id: acpi_hardware_id.into(),
            acpi_unique_id: acpi_unique_id.into(),
        }
    }
}

#[derive(Clone, Debug, Default, AsBytes)]
#[repr(C, packed)]
pub struct PCIDeviceResource {
    // Resource ID 1
    /// The Segment/Bus/Device/Function data of the resource that this controller
    /// is associated with.
    bdf: U32,
    _reserved_resource_id_1: U32,

    // Resource ID 2
    _reserved_resource_id_2: U32,
}

impl PCIDeviceResource {
    pub fn new(bdf: u32) -> Self {
        Self {
            bdf: bdf.into(),
            _reserved_resource_id_1: 0.into(),
            _reserved_resource_id_2: 0.into(),
        }
    }
}

aml_as_bytes!(CacheResource);
aml_as_bytes!(MemoryAffinityStructureResource);
aml_as_bytes!(ACPIDeviceResource);
aml_as_bytes!(PCIDeviceResource);

assert_same_size!(CacheResource, [u8; 12]);
assert_same_size!(MemoryAffinityStructureResource, [u8; 20]);
assert_same_size!(ACPIDeviceResource, [u8; 12]);
assert_same_size!(PCIDeviceResource, [u8; 12]);

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::gas::*;

    #[test]
    fn test_bare_rqsc() {
        let rqsc = RQSC::new(*b"RQSSCC", *b"SOMETHIN", 0xcafe_d00d);
        let mut bytes = Vec::new();
        rqsc.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4);
        assert_eq!(bytes[0..4], *b"RQSC");
    }

    #[test]
    fn test_structures() {
        let mut rqsc = RQSC::new(*b"RQSSCC", *b"SOMETHIN", 0xcafe_d00d);

        // Empty QoS Controller, 28 bytes
        rqsc.add_controller(QoSController::new(
            ControllerType::Capacity,
            gas::GAS::new(
                AddressSpace::SystemMemory,
                64,
                0,
                AccessSize::QwordAccess,
                0x0123_4567_89ab_cdef,
            ),
            0x4242_4242,
            0x3737_3737,
            0,
        ));

        let mut bytes = Vec::new();
        rqsc.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));

        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4 + 28);
        assert_eq!(bytes[0..4], *b"RQSC");

        let mut controller = QoSController::new(
            ControllerType::Capacity,
            gas::GAS::new(
                AddressSpace::SystemMemory,
                64,
                0,
                AccessSize::QwordAccess,
                0x0123_4567_89ab_cdef,
            ),
            0x4242_4242,
            0x3737_3737,
            0,
        );

        // 28 bytes
        controller.add_resource(ResourceStructure::new(
            ResourceType::Cache,
            0,
            ResourceID::MemoryAffinityStructure(MemoryAffinityStructureResource::new(
                0x2468, 0x1357,
            )),
        ));

        // 20 bytes
        controller.add_resource(ResourceStructure::new(
            ResourceType::Cache,
            0,
            ResourceID::Cache(CacheResource::new(0)),
        ));

        // 24 bytes
        controller.add_resource(ResourceStructure::new(
            ResourceType::Cache,
            0,
            ResourceID::VendorSpecific(
                0xFF,
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            ),
        ));

        rqsc.add_controller(controller);

        let mut bytes = Vec::new();
        rqsc.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));

        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4 + 28 + 28 + 28 + 20 + 24);
        assert_eq!(bytes[0..4], *b"RQSC");
    }
}