// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use zerocopy::{byteorder, byteorder::LE, AsBytes};

extern crate alloc;

use crate::{aml_as_bytes, Aml, AmlSink, Checksum, TableHeader};

type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

/// The Boot Error Record Table is used to point to a range of
/// firmware-reserved memory that is used to store details of any
/// unhandled errors that occurred in the previous boot. The format of
/// the Boot Error Region follows that of an `Error Status Block`.

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, AsBytes)]
pub struct BERT {
    header: TableHeader,
    error_region_length: U32,
    error_region_base: U64,
}

impl BERT {
    pub fn new(
        oem_id: [u8; 6],
        oem_table_id: [u8; 8],
        oem_revision: u32,
        error_region_length: u32,
        error_region_base: u64,
    ) -> Self {
        let header = TableHeader {
            signature: *b"BERT",
            length: (TableHeader::len() as u32 + 12).into(),
            revision: 1,
            checksum: 0,
            oem_id,
            oem_table_id,
            oem_revision: oem_revision.into(),
            creator_id: crate::CREATOR_ID,
            creator_revision: crate::CREATOR_REVISION,
        };

        let mut bert = Self {
            header,
            error_region_length: error_region_length.into(),
            error_region_base: error_region_base.into(),
        };

        let mut cksum = Checksum::default();
        cksum.append(bert.as_bytes());
        bert.header.checksum = cksum.value();
        bert
    }
}

aml_as_bytes!(BERT);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_bert() {
        let bert = BERT::new(
            *b"BERRTT",
            *b"SOMETHIN",
            0xcafe_d00d,
            0x1020_3040,
            0x5060_7080_90a0_b0c0,
        );

        let mut bytes = Vec::new();
        bert.to_aml_bytes(&mut bytes);
        let sum = bytes.iter().fold(0u8, |acc, x| acc.wrapping_add(*x));
        assert_eq!(sum, 0);
    }
}