// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{aml_as_bytes, gas, gas::GAS, mutable_setter, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(u16)]
enum HestStructureType {
    PcieAerRootPort = 6,
    PcieAerDevice = 7,
    PcieAerBridge = 8,
    GenericHardware = 9,
    GenericHardwareV2 = 10,
}

pub struct HEST {
    header: TableHeader,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
}

impl HEST {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"HEST",
            length: (TableHeader::len() as u32 + core::mem::size_of::<u32>() as u32).into(),
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

        // The HEST keeps a count of how many structures are
        // contained within it.
        self.checksum.add(1);
        self.header.checksum = self.checksum.value();
    }

    pub fn add_structure<T>(&mut self, t: T)
    where
        T: Aml + AsBytes + Clone + 'static,
    {
        self.update_header(t.as_bytes());
        self.structures.push(Box::new(t));
    }
}

impl Aml for HEST {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        self.header.to_aml_bytes(sink);

        sink.dword(self.structures.len() as u32);

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

/// PCIe root ports may implement Advanced Error Reporting support.
/// This structure contains information for configuring AER support on
/// a given root port.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct PcieAerRootPort {
    r#type: U16,
    source_id: U16,
    _reserved0: U16,
    flags: u8,
    enabled: u8,
    num_records: U32,
    max_sections: U32,
    bus: U32,
    device: U16,
    function: U16,
    device_control: U16,
    _reserved1: U16,
    uncorrectable_error_mask: U32,
    uncorrectable_error_severity: U32,
    correctable_error_mask: U32,
    aer_cap_ctrl: U32,
    root_error_command: U32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum FirmwareFirst {
    Disabled = 0,
    Enabled = 1,
}

pub struct PciDevice {
    bus: u8,
    device: u8,
    function: u8,
}

impl PciDevice {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        assert!(device < 32);
        assert!(function < 8);
        Self {
            bus,
            device,
            function,
        }
    }
}

impl PcieAerRootPort {
    const FLAG_GLOBAL: u8 = 1 << 1;

    #[cfg(test)]
    fn len() -> usize {
        48
    }

    pub fn new_global() -> Self {
        Self {
            r#type: (HestStructureType::PcieAerRootPort as u16).into(),
            flags: Self::FLAG_GLOBAL,
            ..Default::default()
        }
    }

    pub fn new_root_port(ff: FirmwareFirst, device: PciDevice) -> Self {
        Self {
            r#type: (HestStructureType::PcieAerRootPort as u16).into(),
            flags: ff as u8,
            bus: (device.bus as u32).into(),
            device: (device.device as u16).into(),
            function: (device.function as u16).into(),
            ..Default::default()
        }
    }

    mutable_setter!(num_records, u32);
    mutable_setter!(max_sections, u32);
    mutable_setter!(device_control, u16);
    mutable_setter!(uncorrectable_error_mask, u32);
    mutable_setter!(uncorrectable_error_severity, u32);
    mutable_setter!(correctable_error_mask, u32);
    mutable_setter!(aer_cap_ctrl, u32);
    mutable_setter!(root_error_command, u32);
}

aml_as_bytes!(PcieAerRootPort);

/// PCIe devices may implement AER support. This structure contains
/// information OSPM needs to configure AER support on a given PCIe
/// device.  As with the root port structure, there is a GLOBAL flag
/// which indiciates the entry applies to all PCIe endpoints,
/// otherwise there should be one entry for each device that supports
/// AER.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct PcieAerDevice {
    r#type: U16,
    source_id: U16,
    _reserved0: U16,
    flags: u8,
    enabled: u8,
    num_records: U32,
    max_sections: U32,
    bus: U32,
    device: U16,
    function: U16,
    device_control: U16,
    _reserved1: U16,
    uncorrectable_error_mask: U32,
    uncorrectable_error_severity: U32,
    correctable_error_mask: U32,
    aer_cap_ctrl: U32,
}

impl PcieAerDevice {
    const FLAG_GLOBAL: u8 = 1 << 1;

    #[cfg(test)]
    fn len() -> usize {
        44
    }

    pub fn new_global() -> Self {
        Self {
            r#type: (HestStructureType::PcieAerDevice as u16).into(),
            flags: Self::FLAG_GLOBAL,
            ..Default::default()
        }
    }

    pub fn new_root_port(ff: FirmwareFirst, device: PciDevice) -> Self {
        Self {
            r#type: (HestStructureType::PcieAerDevice as u16).into(),
            flags: ff as u8,
            bus: (device.bus as u32).into(),
            device: (device.device as u16).into(),
            function: (device.function as u16).into(),
            ..Default::default()
        }
    }

    mutable_setter!(num_records, u32);
    mutable_setter!(max_sections, u32);
    mutable_setter!(device_control, u16);
    mutable_setter!(uncorrectable_error_mask, u32);
    mutable_setter!(uncorrectable_error_severity, u32);
    mutable_setter!(correctable_error_mask, u32);
    mutable_setter!(aer_cap_ctrl, u32);
}

aml_as_bytes!(PcieAerDevice);

/// PCIe/PCI-X bridges that support AER implement fields that control
/// the behavior of how errors are reported across the bridge.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct PcieAerBridge {
    r#type: U16,
    source_id: U16,
    _reserved0: U16,
    flags: u8,
    enabled: u8,
    num_records: U32,
    max_sections: U32,
    bus: U32,
    device: U16,
    function: U16,
    device_control: U16,
    _reserved1: U16,
    uncorrectable_error_mask: U32,
    uncorrectable_error_severity: U32,
    correctable_error_mask: U32,
    aer_cap_ctrl: U32,
    secondary_uncorrectable_error_mask: U32,
    secondary_uncorrectable_error_severity: U32,
    secondary_aer_cap_ctrl: U32,
}

impl PcieAerBridge {
    const FLAG_GLOBAL: u8 = 1 << 1;

    #[cfg(test)]
    fn len() -> usize {
        56
    }

    pub fn new_global() -> Self {
        Self {
            r#type: (HestStructureType::PcieAerBridge as u16).into(),
            flags: Self::FLAG_GLOBAL,
            ..Default::default()
        }
    }

    pub fn new_bridge(ff: FirmwareFirst, device: PciDevice) -> Self {
        Self {
            r#type: (HestStructureType::PcieAerBridge as u16).into(),
            flags: ff as u8,
            bus: (device.bus as u32).into(),
            device: (device.device as u16).into(),
            function: (device.function as u16).into(),
            ..Default::default()
        }
    }

    mutable_setter!(num_records, u32);
    mutable_setter!(max_sections, u32);
    mutable_setter!(device_control, u16);
    mutable_setter!(uncorrectable_error_mask, u32);
    mutable_setter!(uncorrectable_error_severity, u32);
    mutable_setter!(correctable_error_mask, u32);
    mutable_setter!(aer_cap_ctrl, u32);
    mutable_setter!(secondary_uncorrectable_error_mask, u32);
    mutable_setter!(secondary_uncorrectable_error_severity, u32);
    mutable_setter!(secondary_aer_cap_ctrl, u32);
}

aml_as_bytes!(PcieAerBridge);

/// The platform may describe a generic hardware error source using
/// this structure.  It either uses a non-standard notification
/// mechanism or uses a non-standard format for error reporting.
/// Because of the non-standard interface, OSPM does not have support
/// for configure and control operations, therefore the error source
/// must be configured by firmware during boot.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct GenericHardwareSource {
    r#type: U16,
    source_id: U16,
    related_source_id: U16,
    _flags: u8, // reserved
    enabled: u8,
    num_records: U32,
    max_sections: U32,
    max_raw_length: U32,
    error_status_address: GAS,
    notification: NotificationStructure,
    error_status_block_len: U32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum EnabledStatus {
    Disabled = 0,
    Enabled = 1,
}

impl GenericHardwareSource {
    pub fn new(source_id: u16, enabled: EnabledStatus) -> Self {
        Self {
            r#type: (HestStructureType::GenericHardware as u16).into(),
            source_id: source_id.into(),
            related_source_id: 0xffff.into(),
            _flags: 0,
            enabled: enabled as u8,
            ..Default::default()
        }
    }

    mutable_setter!(num_records, u32);
    mutable_setter!(max_sections, u32);
    mutable_setter!(max_raw_length, u32);
    mutable_setter!(error_status_address, GAS);
    mutable_setter!(notification, NotificationStructure);
    mutable_setter!(error_status_block_len, u32);
}

aml_as_bytes!(GenericHardwareSource);

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct NotificationStructure {
    r#type: NotificationType,
    length: u8,
    conf_write_en: U16,
    poll_interval_ms: U32,
    vector: U32,
    polling_threshold_value: U32,
    polling_threshold_window_ms: U32,
    error_threshold_value: U32,
    error_threshold_window_ms: U32,
}

impl NotificationStructure {
    pub fn new(r#type: NotificationType) -> Self {
        Self {
            r#type,
            length: 28,
            ..Default::default()
        }
    }

    mutable_setter!(conf_write_en, u16);
    mutable_setter!(poll_interval_ms, u32);
    mutable_setter!(vector, u32);
    mutable_setter!(polling_threshold_value, u32);
    mutable_setter!(polling_threshold_window_ms, u32);
    mutable_setter!(error_threshold_value, u32);
    mutable_setter!(error_threshold_window_ms, u32);
}

aml_as_bytes!(NotificationStructure);

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub enum NotificationType {
    #[default]
    Polled = 0,
    ExternalIrq = 1,
    LocalIrq = 2,
    Sci = 3,
    Nmi = 4,
    Cmci = 5,
    Mce = 6,
    GpioSignal = 7,
    Armv8Sea = 8,
    Armv8Sei = 9,
    ExternalGsiv = 10,
    SoftwareException = 11,
    RiscvSupervisorSoftwareEvent = 12,
    RiscvLowPriorityRasInterrupt = 13,
    RiscvHighPriorityRasInterrupt = 14,
    RiscvHardwareErrorException = 15,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, AsBytes, Default)]
pub enum ErrorSeverity {
    Recoverable = 0,
    Fatal = 1,
    Correctable = 2,
    #[default]
    None = 3,
}

/// This structure contains error status information from a given
/// generic error source. OSPM provides an error handler that will
/// present this information to the OS.
pub struct GenericErrorStatus {
    status: u32,
    raw_data_offset: u32,
    raw_data_length: u32,
    generic_data_length: u32,
    severity: ErrorSeverity,
    entries: Vec<Box<dyn Aml>>,
}

impl GenericErrorStatus {
    pub fn new(correctable_count: u32, uncorrectable_count: u32, severity: ErrorSeverity) -> Self {
        let mut status: u32 = 0;
        status |= match correctable_count {
            1 => 1 << 1,
            n if n > 1 => 1 << 3,
            _ => 0,
        };

        status |= match uncorrectable_count {
            1 => 1 << 0,
            n if n > 1 => 1 << 2,
            _ => 0,
        };

        Self {
            status,
            raw_data_length: 0,
            raw_data_offset: 0,
            generic_data_length: 0,
            severity,
            entries: Vec::new(),
        }
    }
}

impl Aml for GenericErrorStatus {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.dword(self.status);
        sink.dword(self.raw_data_offset);
        sink.dword(self.raw_data_length);
        sink.dword(self.generic_data_length);
        sink.dword(self.severity as u32);
        for entry in &self.entries {
            entry.to_aml_bytes(sink);
        }
    }
}

/// This is starting to get into UEFI territory; for some reason the
/// ACPI spec here shells out to the UEFI specification for defining
/// these error records, to wit: "see the definition of Section
/// Descriptors in the UEFI Specification appendix for the Common
/// Platform Error Record."
#[derive(Default)]
pub struct GenericErrorData {
    pub section_type: u16,
    pub severity: ErrorSeverity,
    pub revision: u16,
    pub validation: u8,
    pub flags: u8,
    pub error_data_length: u32,
    pub fru_id: [u8; 16],
    pub fru_text: [u8; 20],
    pub timestamp: [u8; 8],
    // The information contained in this field must match one of the error
    // record section types defined in the UEFI specification appendix, "Common
    // Platform Error Record."
    //
    // For now, this library does not implement the required
    // structures from the CPER. More information can be found at:
    // https://uefi.org/specs/UEFI/2.10/Apx_N_Common_Platform_Error_Record.html
    data: Vec<Box<dyn Aml>>,
}

impl GenericErrorData {
    pub fn new(severity: ErrorSeverity) -> Self {
        Self {
            severity,
            data: Vec::new(),
            ..Default::default()
        }
    }

    pub fn add_data(&mut self, data: Box<dyn Aml>) {
        self.data.push(data);
    }
}

impl Aml for GenericErrorData {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        sink.word(self.section_type);
        sink.dword(self.severity as u32);
        sink.word(self.revision);
        sink.byte(self.validation);
        sink.byte(self.flags);
        sink.dword(self.error_data_length);
        sink.vec(&self.fru_id);
        sink.vec(&self.fru_text);
        sink.vec(&self.timestamp);
        for aml in &self.data {
            aml.to_aml_bytes(sink);
        }
    }
}

/// This is an extension to the Generic Hardware Source structure above,
/// for HW-reduced platforms that rely on "RAS controllers" to generate generic
/// error records.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct GenericHardwareSourceV2 {
    r#type: U16,
    source_id: U16,
    related_source_id: U16,
    _flags: u8, // reserved
    enabled: u8,
    num_records: U32,
    max_sections: U32,
    max_raw_length: U32,
    error_status_address: GAS,
    notification: NotificationStructure,
    error_status_block_len: U32,
    read_ack_register: gas::GAS,
    read_ack_preserve: U64,
    read_ack_write: U64,
}

impl GenericHardwareSourceV2 {
    pub fn new(source_id: u16, enabled: EnabledStatus) -> Self {
        Self {
            r#type: (HestStructureType::GenericHardwareV2 as u16).into(),
            source_id: source_id.into(),
            related_source_id: 0xffff.into(),
            _flags: 0,
            enabled: enabled as u8,
            ..Default::default()
        }
    }

    mutable_setter!(num_records, u32);
    mutable_setter!(max_sections, u32);
    mutable_setter!(max_raw_length, u32);
    mutable_setter!(error_status_address, GAS);
    mutable_setter!(notification, NotificationStructure);
    mutable_setter!(error_status_block_len, u32);
    mutable_setter!(read_ack_register, gas::GAS);
    mutable_setter!(read_ack_preserve, u64);
    mutable_setter!(read_ack_write, u64);
}

aml_as_bytes!(GenericHardwareSourceV2);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gas::*;

    #[test]
    fn test_hest() {
        let hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4);
        assert_eq!(bytes[0..4], *b"HEST");
    }

    #[test]
    fn test_hest_pcirp() {
        let mut hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        hest.add_structure(
            PcieAerRootPort::new_global()
                .num_records(128)
                .max_sections(0x1000)
                .device_control(0x1080)
                .uncorrectable_error_mask(0xcafe)
                .uncorrectable_error_severity(0xd00d)
                .correctable_error_mask(0xbeef)
                .aer_cap_ctrl(0xdeed)
                .root_error_command(0xbaad),
        );

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4 + PcieAerRootPort::len());
        assert_eq!(bytes[0..4], *b"HEST");
    }

    #[test]
    fn test_hest_pcirp_individual() {
        let mut hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        hest.add_structure(
            PcieAerDevice::new_root_port(FirmwareFirst::Enabled, PciDevice::new(0xff, 0x1f, 0x7))
                .num_records(128)
                .max_sections(0x1000)
                .device_control(0x1080)
                .uncorrectable_error_mask(0xcafe)
                .uncorrectable_error_severity(0xd00d)
                .correctable_error_mask(0xbeef)
                .aer_cap_ctrl(0xdeed),
        );

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4 + PcieAerDevice::len());
        assert_eq!(bytes[0..4], *b"HEST");
    }

    #[test]
    fn test_hest_pcirp_bridge() {
        let mut hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        hest.add_structure(
            PcieAerBridge::new_bridge(FirmwareFirst::Disabled, PciDevice::new(0xef, 0x1e, 0x5))
                .num_records(127)
                .max_sections(0x1004)
                .device_control(0x1234)
                .uncorrectable_error_mask(0xcafe)
                .uncorrectable_error_severity(0xd00d)
                .correctable_error_mask(0xbeef)
                .aer_cap_ctrl(0xdeed)
                .secondary_aer_cap_ctrl(0xb157),
        );

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), TableHeader::len() + 4 + PcieAerBridge::len());
        assert_eq!(bytes[0..4], *b"HEST");
    }

    #[test]
    fn test_hest_generic_hardware() {
        let mut hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        hest.add_structure(
            GenericHardwareSource::new(0x1234, EnabledStatus::Enabled)
                .num_records(0x0123_4567)
                .max_sections(0x89ab_cdef)
                .max_raw_length(0x0123_4567)
                .error_status_address(GAS::new(
                    AddressSpace::PciBarTarget,
                    32,
                    16,
                    AccessSize::DwordAccess,
                    0x89ab_cdef,
                ))
                .notification(
                    NotificationStructure::new(NotificationType::Nmi)
                        .conf_write_en(0x1234)
                        .poll_interval_ms(0x5678_90ab)
                        .vector(0xcdef_1234)
                        .polling_threshold_value(0x5678_9abc)
                        .polling_threshold_window_ms(0xdef0_1234)
                        .error_threshold_value(0x5678_9abc)
                        .error_threshold_window_ms(0xdef0_1234),
                )
                .error_status_block_len(0xdead_beef),
        );

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes[0..4], *b"HEST");
    }

    #[test]
    fn test_hest_generic_hardware_v2() {
        let mut hest = HEST::new(*b"HESSTT", *b"SOMETHIN", 0xcafe_d00d);

        hest.add_structure(
            GenericHardwareSourceV2::new(0x1234, EnabledStatus::Enabled)
                .num_records(0x0123_4567)
                .max_sections(0x89ab_cdef)
                .max_raw_length(0x0123_4567)
                .error_status_address(GAS::new(
                    AddressSpace::PciBarTarget,
                    32,
                    16,
                    AccessSize::DwordAccess,
                    0x89ab_cdef,
                ))
                .notification(
                    NotificationStructure::new(NotificationType::Polled)
                        .conf_write_en(0x1234)
                        .poll_interval_ms(0x5678_90ab)
                        .vector(0)
                        .polling_threshold_value(0x5678_9abc)
                        .polling_threshold_window_ms(0xdef0_1234)
                        .error_threshold_value(0x5678_9abc)
                        .error_threshold_window_ms(0xdef0_1234),
                )
                .error_status_block_len(0xdead_beef)
                .read_ack_register(GAS::new(
                    AddressSpace::SystemMemory,
                    64,
                    255,
                    AccessSize::QwordAccess,
                    0x1234_0123_4567,
                ))
                .read_ack_preserve(0x9876_4321_0123_4567)
                .read_ack_write(0x9876_4321_0123_4567),
        );

        let mut bytes = Vec::new();
        hest.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes[0..4], *b"HEST");
    }
}