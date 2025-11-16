// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;

use crate::{gas::GAS, Aml, AmlSink, Checksum, TableHeader};

type U16 = byteorder::U16<LE>;
type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum PlatformClass {
    #[default]
    Client = 0,
    Server = 1,
}

pub struct TpmClient1_2 {
    header: TableHeader,
    log_area_min_len: u32,
    log_area_start_addr: u64,
}

impl TpmClient1_2 {
    pub fn new(
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        log_area_min_len: u32,
        log_area_start_addr: u64,
    ) -> Self {
        let mut header = TableHeader {
            signature: *b"TCPA",
            length: 50.into(),
            revision: 2,
            checksum: 0,
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
        };

        let mut cksum = Checksum::default();
        cksum.append(header.as_bytes());
        cksum.append(log_area_min_len.as_bytes());
        cksum.append(log_area_start_addr.as_bytes());
        header.checksum = cksum.value();

        Self {
            header,
            log_area_min_len,
            log_area_start_addr,
        }
    }
}

impl Aml for TpmClient1_2 {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        self.header.to_aml_bytes(sink);
        sink.word(PlatformClass::Client as u16);
        sink.dword(self.log_area_min_len);
        sink.qword(self.log_area_start_addr);
    }
}

#[derive(Copy, Clone, Default, AsBytes)]
#[repr(C, packed)]
pub struct TpmServer1_2 {
    header: TableHeader,
    platform_class: U16,
    _reserved0: U16,
    log_area_min_len: U64,
    log_area_start_addr: U64,
    tcg_spec_rev_bcd: [u8; 2],
    device_flags: u8,
    interrupt_flags: u8,
    gpe: u8,
    _reserved1: [u8; 3],
    gsi: U32,
    base_addr: GAS,
    _reserved2: U32,
    tpm_config_addr: GAS,
    pci_segment: u8,
    pci_bus: u8,
    pci_device: u8,
    pci_function: u8,
}

impl TpmServer1_2 {
    pub fn new(oem_id: [u8; 6], oem_table_id: [u8; 8], oem_revision: u32) -> Self {
        let mut header = TableHeader {
            signature: *b"TCPA",
            length: 100.into(),
            revision: 2,
            checksum: 0,
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
        };
        let tcg_spec_rev_bcd = [1u8, 2];

        let mut cksum = Checksum::default();
        cksum.append(header.as_bytes());
        cksum.append(&tcg_spec_rev_bcd);
        header.checksum = cksum.value();

        Self {
            header,
            tcg_spec_rev_bcd,
            platform_class: (PlatformClass::Server as u16).into(),
            ..Default::default()
        }
    }

    fn update_header(mut self) -> Self {
        self.header.checksum = 0;
        self.header.checksum = crate::generate_checksum(self.as_bytes());
        self
    }

    pub fn log_area(mut self, log_area_min_len: u64, log_area_start_addr: u64) -> Self {
        self.log_area_min_len = log_area_min_len.into();
        self.log_area_start_addr = log_area_start_addr.into();
        self.update_header()
    }

    pub fn active_low(mut self) -> Self {
        self.interrupt_flags |= 1 << 1;
        self.update_header()
    }

    pub fn edge_triggered(mut self) -> Self {
        self.interrupt_flags |= 1 << 0;
        self.update_header()
    }

    pub fn sci_gpe(mut self, sci_gpe_bit: u8) -> Self {
        self.gpe = sci_gpe_bit;
        self.interrupt_flags |= 1 << 2;
        self.update_header()
    }

    pub fn gsi(mut self, gsi: u32) -> Self {
        self.gsi = gsi.into();
        self.interrupt_flags |= 1 << 3;
        self.update_header()
    }

    pub fn bus_is_pnp(mut self) -> Self {
        self.device_flags |= 1 << 1;
        self.update_header()
    }

    pub fn pci_sbdf(mut self, segment: u8, bus: u8, device: u8, function: u8) -> Self {
        assert!(device < 32);
        assert!(function < 8);

        self.pci_segment = segment;
        self.pci_bus = bus;
        self.pci_device = device;
        self.pci_function = function;
        self.device_flags |= 1 << 0;
        self.update_header()
    }

    pub fn base_addr(mut self, addr: GAS) -> Self {
        self.base_addr = addr;
        self.update_header()
    }

    pub fn config_addr(mut self, addr: GAS) -> Self {
        self.device_flags |= 1 << 2;
        self.tpm_config_addr = addr;
        self.update_header()
    }

    pub fn len() -> usize {
        core::mem::size_of::<Self>()
    }
}

crate::assert_same_size!(TpmServer1_2, [u8; 100]);

impl Aml for TpmServer1_2 {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        for byte in self.as_bytes() {
            sink.byte(*byte);
        }
    }
}

/// TPM 2.0
#[derive(Default)]
pub struct Tpm2 {
    header: TableHeader,
    checksum: Checksum,
    platform_class: PlatformClass,
    crb_or_fifo_base: u64,
    start_method: StartMethod,
    start_method_params: [u8; 12],
    start_method_param_len: usize,
    log_area_min_len: Option<u32>,
    log_area_start_addr: Option<u64>,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum StartMethod {
    #[default]
    LegacyUse = 1,
    AcpiStart = 2,
    Mmio = 6,
    Crb = 7,
    CrbAndAcpiStart = 8,
    CrbAndSmcHvc = 11,
    I2cFifo = 12,
}

impl Tpm2 {
    pub fn new(
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        platform_class: PlatformClass,
        crb_or_fifo_base: u64,
        start_method: StartMethod,
    ) -> Self {
        let mut header = TableHeader {
            signature: *b"TPM2",
            length: 52.into(),
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
        cksum.append((platform_class as u16).as_bytes());
        cksum.append(crb_or_fifo_base.as_bytes());
        cksum.append((start_method as u32).as_bytes());
        header.checksum = cksum.value();

        Self {
            header,
            checksum: cksum,
            platform_class,
            crb_or_fifo_base,
            start_method,
            ..Default::default()
        }
    }

    pub fn set_log_area(&mut self, min_len: u32, base_addr: u64) {
        let old_len = self.header.length.get();
        assert!(old_len == 52);

        // old_len + 4 (min_len) + 8 (base_addr) + 12 (start method params)
        let new_len = old_len + 24;
        self.header.length.set(new_len);

        self.checksum.delete(old_len.as_bytes());
        self.checksum.append(new_len.as_bytes());
        self.checksum.append(min_len.as_bytes());
        self.checksum.append(base_addr.as_bytes());
        self.header.checksum = self.checksum.value();

        self.start_method_param_len = 12;
        self.log_area_min_len = Some(min_len);
        self.log_area_start_addr = Some(base_addr);
    }
}

impl Aml for Tpm2 {
    fn to_aml_bytes(&self, sink: &mut dyn AmlSink) {
        self.header.to_aml_bytes(sink);
        sink.word(self.platform_class as u16);
        sink.word(0); // reserved
        sink.qword(self.crb_or_fifo_base);
        sink.dword(self.start_method as u32);
        for byte in &self.start_method_params[0..self.start_method_param_len] {
            sink.byte(*byte);
        }
        if let Some(laml) = self.log_area_min_len {
            sink.dword(laml);
        }
        if let Some(lasa) = self.log_area_start_addr {
            sink.qword(lasa);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gas;
    use alloc::vec::Vec;

    #[test]
    fn test_client() {
        let client = TpmClient1_2::new(
            *b"FOOBAR",
            *b"CAFEDEAD",
            0xdead_beef,
            0x8000_0000,
            0x1234_5678_9012_3456,
        );

        let mut bytes = Vec::new();
        client.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }

    #[test]
    fn test_server() {
        let server = TpmServer1_2::new(*b"FOOBAR", *b"CAFEDEAD", 0xdead_beef)
            .log_area(0x8000_0000, 0x1234_5678_9012_3456)
            .active_low()
            .edge_triggered()
            .sci_gpe(0x40)
            .gsi(0x80)
            .pci_sbdf(1, 2, 3, 4)
            .config_addr(GAS::new(
                gas::AddressSpace::SystemMemory,
                32,
                0,
                gas::AccessSize::DwordAccess,
                0x8070_6050_4030_2010,
            ))
            .base_addr(GAS::new(
                gas::AddressSpace::SystemIo,
                8,
                0,
                gas::AccessSize::ByteAccess,
                0x8070_6050_4030_2010,
            ));

        let mut bytes = Vec::new();
        server.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 100);
    }

    #[test]
    fn test_tpm2() {
        let mut tpm2 = Tpm2::new(
            *b"FOOBAR",
            *b"CAFEDEAD",
            0xdead_beef,
            PlatformClass::Server,
            0x0123_4567_8901_2345,
            StartMethod::Crb,
        );
        let mut bytes = Vec::new();
        tpm2.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 52);

        tpm2.set_log_area(0x8070_6050, 0x4030_2010_f0e0_d0c0);

        let mut bytes = Vec::new();
        tpm2.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
        assert_eq!(bytes.len(), 76);
    }
}