// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

use crate::{aml_as_bytes, assert_same_size, mutable_setter, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(u8)]
enum MadtStructureType {
    ProcessorLocalApic = 0x0,
    IoApic = 0x1,
    GicCpuInterface = 0xb,
    GicDistributor = 0xc,
    GicMsiFrame = 0xd,
    GicRedistributor = 0xe,
    GicTranslationService = 0xf,
    RiscvIntc = 0x18,
    RiscvImsic = 0x19,
    RiscvAplic = 0x1a,
    RiscvPlic = 0x1b,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
struct Header {
    table_header: TableHeader,
    /// Must be ignored by OSPM for RISC-V
    local_interrupt_controller_address: U32,
    flags: U32,
}

impl Header {
    fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

pub struct MADT {
    header: Header,
    checksum: Checksum,
    structures: Vec<Box<dyn Aml>>,
    has_imsic: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterruptController {
    Riscv,
    Address(u32),
}

impl MADT {
    pub fn new(
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        int: LocalInterruptController,
    ) -> Self {
        let mut header = Header {
            table_header: TableHeader {
                signature: *b"APIC",
                length: (Header::len() as u32).into(),
                revision: 1,
                checksum: 0,
                oem_id,
                oem_table_id,
                oem_revision: oem_revision.into(),
                creator_id: crate::CREATOR_ID,
                creator_revision: crate::CREATOR_REVISION,
            },
            local_interrupt_controller_address: match int {
                LocalInterruptController::Riscv => 0,
                LocalInterruptController::Address(addr) => addr,
            }
            .into(),
            flags: 0.into(),
        };

        let mut cksum = Checksum::default();
        cksum.append(header.as_bytes());
        header.table_header.checksum = cksum.value();
        Self {
            header,
            checksum: cksum,
            structures: Vec::new(),
            has_imsic: false,
        }
    }

    fn update_header(&mut self, data: &[u8]) {
        let len = data.len() as u32;
        let old_len = self.header.table_header.length.get();
        let new_len = len + old_len;
        self.header.table_header.length.set(new_len);

        // Remove the bytes from the old length, add the new length
        // and the new data.
        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.append(data);
        self.header.table_header.checksum = self.checksum.value();
    }

    pub fn add_structure<T>(&mut self, t: T)
    where
        T: Aml + AsBytes + Clone + 'static,
    {
        self.update_header(t.as_bytes());
        self.structures.push(Box::new(t));
    }

    pub fn add_imsic(&mut self, imsic: IMSIC) {
        assert!(!self.has_imsic);
        self.add_structure(imsic);
        self.has_imsic = true;
    }
}

impl Aml for MADT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.header.as_bytes() {
            sink.byte(*byte);
        }

        for st in &self.structures {
            st.to_aml_bytes(sink);
        }
    }
}

/// Processor-Local APIC
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct ProcessorLocalApic {
    r#type: u8,
    length: u8,
    processor_uid: u8,
    apic_id: u8,
    flags: U32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum EnabledStatus {
    Disabled = 0,
    Enabled = 1,
    DisabledOnlineCapable = 2,
}

impl ProcessorLocalApic {
    pub fn new(uid: u8, apic_id: u8, enabled: EnabledStatus) -> Self {
        Self {
            r#type: MadtStructureType::ProcessorLocalApic as u8,
            length: 8,
            processor_uid: uid,
            apic_id,
            flags: (enabled as u32).into(),
        }
    }
}

aml_as_bytes!(ProcessorLocalApic);

/// I/O APIC
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct IoApic {
    r#type: u8,
    length: u8,
    io_apic_id: u8,
    _reserved: u8,
    io_apic_addr: U32,
    gsi_base: U32,
}

impl IoApic {
    pub fn new(io_apic_id: u8, io_apic_addr: u32, gsi_base: u32) -> Self {
        Self {
            r#type: MadtStructureType::IoApic as u8,
            length: 12,
            io_apic_id,
            _reserved: 0,
            io_apic_addr: io_apic_addr.into(),
            gsi_base: gsi_base.into(),
        }
    }
}

aml_as_bytes!(IoApic);

/// GIC CPU Interface (GICC)
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct Gicc {
    r#type: u8,
    length: u8,
    _reserved0: U16,
    cpu_interface_number: U32,
    acpi_processor_uid: U32,
    flags: U32,
    parking_protocol_version: U32,
    performance_interrupt: U32,
    parked_address: U64,
    base_address: U64,
    virtual_registers: U64,
    control_block_registers: U64,
    maintenance_interrupt: U32,
    redistributor_base: U64,
    mpidr: U64,
    power_efficiency_class: u8,
    _reserved1: u8,
    overflow_interrupt: U16,
    trbe_interrupt: U16,
}

#[repr(u32)]
enum GiccFlags {
    Enabled = 1 << 0,
    PerformanceInterruptEdgeTriggered = 1 << 1,
    MaintenanceInterruptEdgeTriggered = 1 << 2,
    OnlineCapable = 1 << 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    Edge,
    Level,
}

impl Gicc {
    fn len() -> usize {
        core::mem::size_of::<Self>()
    }

    pub fn new(status: EnabledStatus) -> Self {
        let flags = match status {
            EnabledStatus::Enabled => GiccFlags::Enabled as u32,
            EnabledStatus::Disabled => 0,
            EnabledStatus::DisabledOnlineCapable => GiccFlags::OnlineCapable as u32,
        };

        Self {
            r#type: MadtStructureType::GicCpuInterface as u8,
            length: Self::len() as u8,
            flags: flags.into(),
            ..Default::default()
        }
    }

    pub fn performance_interrupt(mut self, gsi: u32, trigger: Trigger) -> Self {
        if trigger == Trigger::Edge {
            let flags = self.flags.get();
            self.flags
                .set(flags | GiccFlags::PerformanceInterruptEdgeTriggered as u32);
        }
        self.performance_interrupt = gsi.into();
        self
    }

    pub fn maintenance_interrupt(mut self, gsi: u32, trigger: Trigger) -> Self {
        if trigger == Trigger::Edge {
            let flags = self.flags.get();
            self.flags
                .set(flags | GiccFlags::MaintenanceInterruptEdgeTriggered as u32);
        }
        self.maintenance_interrupt = gsi.into();
        self
    }

    mutable_setter!(cpu_interface_number, u32);
    mutable_setter!(acpi_processor_uid, u32);
    mutable_setter!(parking_protocol_version, u32);
    mutable_setter!(parked_address, u64);
    mutable_setter!(base_address, u64);
    mutable_setter!(virtual_registers, u64);
    mutable_setter!(control_block_registers, u64);
    mutable_setter!(redistributor_base, u64);
    mutable_setter!(mpidr, u64);
    mutable_setter!(power_efficiency_class, u8);
    mutable_setter!(overflow_interrupt, u16);
    mutable_setter!(trbe_interrupt, u16);
}

assert_same_size!(Gicc, [u8; 82]);
aml_as_bytes!(Gicc);

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum GicVersion {
    Unspecified = 0,
    GICv1 = 1,
    GICv2 = 2,
    GICv3 = 3,
    GICv4 = 4,
}

/// GIC Distributor (GICD) Structure
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct Gicd {
    r#type: u8,
    length: u8,
    _reserved0: U16,
    gic_id: U32,
    base_addr: U64,
    vector_base: U32,
    gic_version: u8,
    _reserved1: [u8; 3],
}

impl Gicd {
    pub fn new(gic_id: u32, base_addr: u64, version: GicVersion) -> Self {
        Self {
            r#type: MadtStructureType::GicDistributor as u8,
            length: 24,
            _reserved0: 0.into(),
            gic_id: gic_id.into(),
            base_addr: base_addr.into(),
            vector_base: 0.into(),
            gic_version: version as u8,
            _reserved1: [0, 0, 0],
        }
    }
}

assert_same_size!(Gicd, [u8; 24]);
aml_as_bytes!(Gicd);

/// GIC MSI Frame Structure
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct GicMsi {
    r#type: u8,
    length: u8,
    _reserved: U16,
    gic_msi_frame_id: U32,
    base_addr: U64,
    flags: U32,
    spi_count: U16,
    spi_base: U16,
}

impl GicMsi {
    pub fn new() -> Self {
        Self {
            r#type: MadtStructureType::GicMsiFrame as u8,
            length: 24,
            _reserved: 0.into(),
            flags: 1.into(), /* Ignore SPI count and base until set */
            ..Default::default()
        }
    }

    mutable_setter!(gic_msi_frame_id, u32);
    mutable_setter!(base_addr, u64);
    pub fn spi_count_and_base(mut self, spi_count: u16, spi_base: u16) -> Self {
        self.spi_count = spi_count.into();
        self.spi_base = spi_base.into();
        self.flags = 0.into();
        self
    }
}

assert_same_size!(GicMsi, [u8; 24]);
aml_as_bytes!(GicMsi);

/// GIC Redistributor (GICR) Structure
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct Gicr {
    r#type: u8,
    length: u8,
    _reserved: U16,
    discovery_range_base: U64,
    discovery_range_length: U32,
}

impl Gicr {
    pub fn new(discovery_range_base: u64, discovery_range_length: u32) -> Self {
        Self {
            r#type: MadtStructureType::GicRedistributor as u8,
            length: 16,
            _reserved: 0.into(),
            discovery_range_base: discovery_range_base.into(),
            discovery_range_length: discovery_range_length.into(),
        }
    }
}

assert_same_size!(Gicr, [u8; 16]);
aml_as_bytes!(Gicr);

/// GIC Interrupt Translation Service (ITS) Structure
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct GicIts {
    r#type: u8,
    length: u8,
    _reserved0: U16,
    gic_its_id: U32,
    base_addr: U64,
    _reserved1: U32,
}

impl GicIts {
    pub fn new(gic_its_id: u32, base_addr: u64) -> Self {
        Self {
            r#type: MadtStructureType::GicTranslationService as u8,
            length: 20,
            _reserved0: 0.into(),
            gic_its_id: gic_its_id.into(),
            base_addr: base_addr.into(),
            _reserved1: 0.into(),
        }
    }
}

assert_same_size!(GicIts, [u8; 20]);
aml_as_bytes!(GicIts);

/// RISC-V Interrupt Controller (RINTC) structure
/// RISC-V platforms need to have a simple, per-hart interrupt controller
/// available to supervisor mode.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct RINTC {
    r#type: u8,
    length: u8,
    version: u8,
    _reserved: u8,
    flags: U32,
    hart_id: U64,
    acpi_processor_uid: U32,
    ext_int_ctrl_id: U32,
    imsic_base_addr: U64,
    imsic_size: U32,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HartStatus {
    Disabled = 0,
    Enabled = 1,
    OnlineCapable = 2,
}

impl RINTC {
    pub fn new(
        hart_status: HartStatus,
        mhartid: u64,
        acpi_processor_uid: u32,
        ext_int_ctrl_id: u32,
        imsic_base_addr: u64,
        imsic_size: u32,
    ) -> Self {
        Self {
            r#type: MadtStructureType::RiscvIntc as u8,
            length: RINTC::len() as u8,
            version: 1,
            _reserved: 0,
            flags: (hart_status as u32).into(),
            hart_id: mhartid.into(),
            acpi_processor_uid: acpi_processor_uid.into(),
            ext_int_ctrl_id: ext_int_ctrl_id.into(),
            imsic_base_addr: imsic_base_addr.into(),
            imsic_size: imsic_size.into(),
        }
    }

    pub fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

assert_same_size!(RINTC, [u8; 0x24]);
aml_as_bytes!(RINTC);

// Even though IMSIC is a per-processor device, there should be only
// one IMSIC structure present in the MADT for a RISC-V system that
// provides information common across processors. The per-processor
// information will be provided by the RINTC structure.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, AsBytes)]
pub struct IMSIC {
    r#type: u8,
    length: u8,
    version: u8,
    _reserved: [u8; 5],
    // How many interrupt identities are supported by the IMSIC
    // interrupt file in supervisor mode (minimum 63 maximum 2047).
    num_supervisor_interrupt_identities: U16,
    // How many interrupt identities are supported by the IMSIC
    // interrupt file in guest mode (minimum 63 maximum 2047).
    num_guest_interrupt_identities: U16,
    // Number of guest index bits in MSI target address (0 - 7)
    guest_index_bits: u8,
    // Number of hart index bits in the MSI target address (0 - 15)
    hart_index_bits: u8,
    // Number of group index bits in the MSI target address (0 - 7)
    group_index_bits: u8,
    // LSB of the group index bits in the MSI target address (0 - 55)
    group_index_shift: u8,
}

impl IMSIC {
    pub fn new(
        num_supervisor_interrupt_identities: u16,
        num_guest_interrupt_identities: u16,
        guest_index_bits: u8,
        hart_index_bits: u8,
        group_index_bits: u8,
        group_index_shift: u8,
    ) -> Self {
        Self {
            r#type: MadtStructureType::RiscvImsic as u8,
            length: IMSIC::len() as u8,
            version: 1,
            _reserved: [0, 0, 0, 0, 0],
            num_supervisor_interrupt_identities: num_supervisor_interrupt_identities.into(),
            num_guest_interrupt_identities: num_guest_interrupt_identities.into(),
            guest_index_bits,
            hart_index_bits,
            group_index_bits,
            group_index_shift,
        }
    }

    pub fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

assert_same_size!(IMSIC, [u8; 16]);
aml_as_bytes!(IMSIC);

// The RISC-V AIA defines an APLIC for handling wired interrupts on a
// RISC-V platform. In a machine without IMSICs, every RISC-V hart
// accepts interrupts from exactly one APLIC which is the external
// interrupt controller for that hart. RISC-V harts that have IMSICs
// as their external interrupt controllers can receive external
// interrupts only in the form of MSIs. In that case, the role of an
// APLIC is to convert wired interrupts into MSIs for harts.
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct APLIC {
    r#type: u8,
    length: u8,
    version: u8,
    aplic_id: u8,
    flags: U32,
    hardware_id: [u8; 8],
    number_of_idcs: U16,
    total_external_interrupt_sources: U16,
    global_system_interrupt_base: U32,
    aplic_address: U64,
    aplic_size: U32,
}

impl APLIC {
    pub fn new(
        aplic_id: u8,
        hardware_id: [u8; 8],
        number_of_idcs: u16,
        global_system_interrupt_base: u32,
        aplic_address: u64,
        aplic_size: u32,
        total_external_interrupt_sources: u16,
    ) -> Self {
        Self {
            r#type: MadtStructureType::RiscvAplic as u8,
            length: Self::len() as u8,
            version: 1,
            flags: 0.into(),
            aplic_id,
            hardware_id,
            number_of_idcs: number_of_idcs.into(),
            global_system_interrupt_base: global_system_interrupt_base.into(),
            aplic_address: aplic_address.into(),
            aplic_size: aplic_size.into(),
            total_external_interrupt_sources: total_external_interrupt_sources.into(),
        }
    }

    pub fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

assert_same_size!(APLIC, [u8; 36]);
aml_as_bytes!(APLIC);

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct PLIC {
    r#type: u8,
    length: u8,
    version: u8,
    plic_id: u8,
    hardware_id: [u8; 8],
    total_external_interrupt_sources: U16,
    max_priority: U16,
    flags: U32,
    plic_size: U32,
    plic_address: U64,
    global_system_interrupt_base: U32,
}

impl PLIC {
    pub fn new(
        plic_id: u8,
        hardware_id: [u8; 8],
        total_external_interrupt_sources: u16,
        max_priority: u16,
        plic_size: u32,
        plic_address: u64,
        global_system_interrupt_base: u32,
    ) -> Self {
        Self {
            r#type: MadtStructureType::RiscvPlic as u8,
            length: Self::len() as u8,
            version: 1,
            plic_id,
            hardware_id,
            flags: 0.into(),
            total_external_interrupt_sources: total_external_interrupt_sources.into(),
            max_priority: max_priority.into(),
            plic_size: plic_size.into(),
            plic_address: plic_address.into(),
            global_system_interrupt_base: global_system_interrupt_base.into(),
        }
    }

    pub fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

assert_same_size!(PLIC, [u8; 36]);
aml_as_bytes!(PLIC);

#[cfg(test)]
mod tests {
    use super::*;

    fn check_checksum(madt: &MADT) {
        let mut bytes = Vec::new();
        madt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    fn get_size(madt: &MADT) -> usize {
        let mut bytes = Vec::new();
        madt.to_aml_bytes(&mut bytes);
        bytes.len()
    }

    #[test]
    fn test_madt() {
        let madt = MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Riscv,
        );
        check_checksum(&madt);
        assert_eq!(Header::len(), get_size(&madt));
    }

    fn default_madt() -> MADT {
        MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Address(0xfecd_ba90),
        )
    }

    #[test]
    fn test_processor_local_apic() {
        let mut madt = default_madt();

        for i in 0..64 {
            madt.add_structure(ProcessorLocalApic::new(
                i,
                i + 32,
                match i % 3 {
                    0 => EnabledStatus::Enabled,
                    1 => EnabledStatus::Disabled,
                    2 => EnabledStatus::DisabledOnlineCapable,
                    _ => unreachable!(),
                },
            ));
            check_checksum(&madt);
        }
    }

    #[test]
    fn test_ioapic() {
        let mut madt = default_madt();
        for i in 0..64 {
            madt.add_structure(IoApic::new(i, i as u32 * 0x1000, i as u32 * 0x2000));
            check_checksum(&madt);
        }
    }

    #[test]
    fn test_gicc() {
        let mut madt = default_madt();

        let gicc = Gicc::new(EnabledStatus::Enabled)
            .cpu_interface_number(0x1000)
            .acpi_processor_uid(0x2000)
            .parking_protocol_version(0x3000)
            .performance_interrupt(0x4000, Trigger::Edge)
            .parked_address(0x5000)
            .base_address(0x6000)
            .virtual_registers(0x7000)
            .control_block_registers(0x8000)
            .maintenance_interrupt(0x9000, Trigger::Edge)
            .redistributor_base(0xa000)
            .mpidr(0xb000)
            .power_efficiency_class(0xc0)
            .overflow_interrupt(0xd000)
            .trbe_interrupt(0xe000);

        madt.add_structure(gicc);
        check_checksum(&madt);
    }

    #[test]
    fn test_gicd() {
        let mut madt = default_madt();

        let gicd = Gicd::new(0x1020_3040, 0x5060_7080_90a0_b0c0, GicVersion::GICv1);
        madt.add_structure(gicd);
        check_checksum(&madt);
    }

    #[test]
    fn test_rintc() {
        let mut madt = MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Riscv,
        );
        check_checksum(&madt);
        assert_eq!(Header::len(), get_size(&madt));

        for i in 0..128 {
            let rintc = RINTC::new(
                HartStatus::Enabled,
                // mhartid
                42 + i,
                // ACPI UID
                (i + 0x1000) as u32,
                // external interrupt controller id,
                i as u32,
                // imsic base address
                i * 4096 + 0x8000_0000_0000,
                // imsic size
                4096,
            );
            madt.add_structure(rintc);
            check_checksum(&madt);
            assert_eq!(
                Header::len() + RINTC::len() * (i + 1) as usize,
                get_size(&madt)
            );
        }
    }

    #[test]
    fn test_imsic() {
        let mut madt = MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Riscv,
        );
        check_checksum(&madt);
        assert_eq!(Header::len(), get_size(&madt));

        let imsic = IMSIC::new(
            10, /* num_supervisor_interrupt_identities */
            10, /* num_guest_interrupt_identities */
            8,  /* guest_index_bits */
            8,  /* hart_index_bits */
            8,  /* group_index_bits */
            8,  /* group_index_shift */
        );
        madt.add_imsic(imsic);
        check_checksum(&madt);
        assert_eq!(Header::len() + IMSIC::len(), get_size(&madt));
    }

    #[test]
    fn test_aplic() {
        let mut madt = MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Riscv,
        );
        check_checksum(&madt);
        assert_eq!(Header::len(), get_size(&madt));

        for i in 0..2 {
            let aplic = APLIC::new(
                0,                                       /* aplic_id */
                [b'A', b'B', b'C', b'D', b'E', 0, 0, 0], /* hardware_id */
                2,                                       /* number_of_idcs */
                0x8000_0000,                             /* global_system_interrupt_base */
                0x1_0000_0000,                           /* aplic_address */
                0x8192,                                  /* aplic_size */
                767,                                     /* total_external_interrupt_sources */
            );

            madt.add_structure(aplic);
            check_checksum(&madt);
            assert_eq!(Header::len() + APLIC::len() * (i + 1), get_size(&madt));
        }
    }

    #[test]
    fn test_plic() {
        let mut madt = MADT::new(
            *b"FOOBAR",
            *b"DECAFCOF",
            0xdead_beef,
            LocalInterruptController::Riscv,
        );
        check_checksum(&madt);
        assert_eq!(Header::len(), get_size(&madt));

        for i in 0..2 {
            let plic = PLIC::new(
                0,                                       /* plic_id */
                [b'A', b'B', b'C', b'D', b'E', 0, 0, 0], /* hardware_id */
                545,                                     /* total_external_interrupt_sources */
                64,                                      /* max priority */
                0x8000_0000,                             /* global_system_interrupt_base */
                0x4000,                                  /* plic_size */
                0x1000_0000,                             /* plic_address */
            );

            madt.add_structure(plic);
            check_checksum(&madt);
            assert_eq!(Header::len() + PLIC::len() * (i + 1), get_size(&madt));
        }
    }
}