// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use crate::{gas::GAS, Aml, AmlSink};
use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Flags {
    Wbinvd = 1 << 0,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    WbinvdFlush = 1 << 1,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    ProcC1 = 1 << 2,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    PLvl2Up = 1 << 3,
    PwrButton = 1 << 4,
    SlpButton = 1 << 5,
    FixRtc = 1 << 6,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    RtcS4 = 1 << 7,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    TmrValExt = 1 << 8,
    DckCap = 1 << 9,
    ResetRegSup = 1 << 10,
    SealedCase = 1 << 11,
    Headless = 1 << 12,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    CpuSwSlp = 1 << 13,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    PciExpWak = 1 << 14,
    UsePlatformClock = 1 << 15,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    S4RtcStsValid = 1 << 16,
    // NOTE: This bit is ignored on HW_REDUCED platforms
    RemotePowerOnCapable = 1 << 17,
    ForceApicClusterModel = 1 << 18,
    ForceApicPhysicalDestinationMode = 1 << 19,
    HwReducedAcpi = 1 << 20,
    LowPowerS0IdleCapable = 1 << 21,
    PersistentCpuCachesNotReported = 0 << 22,
    PersistentCpuCachesNotPersistent = 1 << 22,
    PersistentCpuCachesArePersistent = 2 << 22,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum PmProfile {
    Unspecified = 0,
    Desktop = 1,
    Mobile = 2,
    Workstation = 3,
    EnterpriseServer = 4,
    SohoServer = 5,
    AppliancePc = 6,
    PerformanceServer = 7,
    Tablet = 8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Default, AsBytes)]
pub struct FADTBuilder {
    pub signature: [u8; 4],
    pub length: U32,
    pub major_version: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    // Note: oem_table_id must match the OEM Table ID in the RSDT
    pub oem_table_id: [u8; 8],
    pub oem_revision: U32,
    pub creator_id: [u8; 4],
    pub creator_revision: [u8; 4],
    pub firmware_ctrl: U32,
    pub dsdt: U32,
    _reserved0: u8,
    pub preferred_pm_profile: u8,
    // NOTE: For HW_REDUCED platforms, sci_int through century are ignored
    pub sci_int: U16,
    pub smi_cmd: U32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_cnt: u8,
    pub pm1a_evt_blk: U32,
    pub pm1b_evt_blk: U32,
    pub pm1a_cnt_blk: U32,
    pub pm1b_cnt_blk: U32,
    pub pm2_cnt_blk: U32,
    pub pm_tmr_blk: U32,
    pub gpe0_blk: U32,
    pub gpe1_blk: U32,
    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: U16,
    pub p_lvl3_lat: U16,
    pub flush_size: U16,
    pub flush_stride: U16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alrm: u8,
    pub mon_alrm: u8,
    pub century: u8,
    pub iapc_boot_arch: U16,
    _reserved1: u8,
    pub flags: U32,
    pub reset_reg: GAS,
    pub reset_value: u8,
    pub arm_boot_arch: U16,
    pub fadt_minor_version: u8,
    pub x_firmware_ctrl: U64,
    pub x_dsdt: U64,
    // NOTE: for HW_REDUCED platforms, x_pm1a_evt_blk through x_gpe1_blk are ignored
    pub x_pm1a_evt_blk: GAS,
    pub x_pm1b_evt_blk: GAS,
    pub x_pm1a_cnt_blk: GAS,
    pub x_pm1b_cnt_blk: GAS,
    pub x_pm2_cnt_blk: GAS,
    pub x_pm_tmr_blk: GAS,
    pub x_gpe0_blk: GAS,
    pub x_gpe1_blk: GAS,
    pub sleep_control_reg: GAS,
    pub sleep_status_reg: GAS,
    pub hypervisor_vendor_identity: U64,
}

pub struct FADT {
    table: FADTBuilder,
}

impl FADT {
    pub fn len() -> usize {
        core::mem::size_of::<FADTBuilder>()
    }
}

impl Aml for FADT {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.table.as_bytes() {
            sink.byte(*byte);
        }
    }
}

impl FADTBuilder {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        Self {
            signature: *b"FACP",
            major_version: 6,      // TODO: should come from ACPI spec version #
            fadt_minor_version: 5, // TODO: should come from ACPI spec version #
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
            length: (FADT::len() as u32).into(),
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            ..Default::default()
        }
    }

    pub fn finalize(mut self) -> FADT {
        self.update_checksum();
        FADT { table: self }
    }

    fn update_checksum(&mut self) {
        self.checksum = 0;
        let checksum = super::generate_checksum(self.as_bytes());
        self.checksum = checksum;
    }

    pub fn dsdt_32(mut self, dsdt_physical_addr: u32) -> Self {
        self.dsdt = dsdt_physical_addr.into();
        self.x_dsdt = 0.into();
        self
    }

    pub fn dsdt_64(mut self, dsdt_physical_addr: u64) -> Self {
        self.dsdt = 0.into();
        self.x_dsdt = dsdt_physical_addr.into();
        self
    }

    pub fn firmware_ctrl_32(mut self, facs_physical_addr: u32) -> Self {
        self.firmware_ctrl = facs_physical_addr.into();
        self.x_firmware_ctrl = 0.into();
        self
    }

    pub fn firmware_ctrl_64(mut self, facs_physical_addr: u64) -> Self {
        self.firmware_ctrl = 0.into();
        self.x_firmware_ctrl = facs_physical_addr.into();
        self
    }

    pub fn acpi_enable(mut self) -> Self {
        self.acpi_enable = 1;
        self.acpi_disable = 0;
        self
    }

    pub fn acpi_disable(mut self) -> Self {
        self.acpi_enable = 0;
        self.acpi_disable = 1;
        self
    }

    pub fn flag(mut self, flags: Flags) -> Self {
        self.flags = (u32::from(self.flags) | flags as u32).into();
        self
    }

    pub fn gpe_info(
        mut self,
        gpe0_blk: u32,
        gpe1_blk: u32,
        gpe0_blk_len: u8,
        gpe1_blk_len: u8,
        gpe1_base: u8,
    ) -> Self {
        self.gpe0_blk = gpe0_blk.into();
        self.gpe1_blk = gpe1_blk.into();
        self.gpe0_blk_len = gpe0_blk_len;
        self.gpe1_blk_len = gpe1_blk_len;
        self.gpe1_base = gpe1_base;
        self
    }

    pub fn preferred_pm_profile(mut self, profile: PmProfile) -> Self {
        self.preferred_pm_profile = profile as u8;
        self
    }
}

#[cfg(test)]
mod test {
    use super::{FADTBuilder, Flags, PmProfile};
    use crate::Aml;
    use alloc::vec::Vec;

    #[test]
    fn test_fadt() {
        let mut bytes = Vec::new();
        let fadt = FADTBuilder::new(*b"TEST__", *b"TESTTEST", 0x4237_5689)
            .acpi_enable()
            .dsdt_64(0x8000_0000_0000)
            .firmware_ctrl_64(0x8001_0000_0000)
            .flag(Flags::Wbinvd)
            .flag(Flags::TmrValExt)
            .flag(Flags::HwReducedAcpi)
            .flag(Flags::Headless)
            .gpe_info(0x1800, 0x1900, 0x20, 0x20, 0x20)
            .preferred_pm_profile(PmProfile::EnterpriseServer)
            .finalize();
        fadt.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}