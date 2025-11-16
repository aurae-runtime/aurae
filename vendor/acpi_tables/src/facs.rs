// Copyright 2023 Rivos, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//

use crate::{Aml, AmlSink};
use zerocopy::{byteorder, byteorder::LE, AsBytes};

type U32 = byteorder::U32<LE>;
type U64 = byteorder::U64<LE>;

#[repr(C, packed)]
#[derive(Clone, Copy, Default, AsBytes)]
pub struct FACS {
    pub signature: [u8; 4],
    pub length: U32,
    pub hardware_signature: U32,
    pub waking: U32,
    pub lock: U32,
    pub flags: U32,
    pub x_waking: U64,
    pub version: u8,
    _reserved1: [u8; 3],
    pub ospm_flags: U32,
    _reserved2: [u8; 24],
}

impl FACS {
    pub fn new() -> Self {
        FACS {
            signature: *b"FACS",
            length: (core::mem::size_of::<FACS>() as u32).into(),
            hardware_signature: 0.into(),
            waking: 0.into(),
            lock: 0.into(),
            flags: 0.into(),
            x_waking: 0.into(),
            version: 1,
            _reserved1: [0; 3],
            ospm_flags: 0.into(),
            _reserved2: [0; 24],
        }
    }

    pub fn len() -> usize {
        core::mem::size_of::<FACS>()
    }
}

crate::aml_as_bytes!(FACS);