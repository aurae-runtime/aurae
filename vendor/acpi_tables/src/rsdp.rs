// Copyright © 2019 Intel Corporation
// Copyright © 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

use crate::{aml_as_bytes, Aml, AmlSink};

#[repr(C, packed)]
#[derive(Clone, Copy, Default, AsBytes)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    _rsdt_addr: U32,
    pub length: U32,
    pub xsdt_addr: U64,
    pub extended_checksum: u8,
    _reserved: [u8; 3],
}

impl Rsdp {
    pub fn new(oem_id: [u8; 6], xsdt_addr: u64) -> Self {
        let mut rsdp = Rsdp {
            signature: *b"RSD PTR ",
            checksum: 0,
            oem_id,
            revision: 2,
            _rsdt_addr: 0.into(),
            length: (core::mem::size_of::<Rsdp>() as u32).into(),
            xsdt_addr: xsdt_addr.into(),
            extended_checksum: 0,
            _reserved: [0; 3],
        };

        rsdp.checksum = super::generate_checksum(&rsdp.as_bytes()[0..20]);
        rsdp.extended_checksum = super::generate_checksum(rsdp.as_bytes());
        rsdp
    }

    pub fn len() -> usize {
        core::mem::size_of::<Rsdp>()
    }
}

aml_as_bytes!(Rsdp);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsdp() {
        let rsdp = Rsdp::new(*b"CHYPER", 0xdead_beef);
        let sum = rsdp
            .as_bytes()
            .iter()
            .fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}