// Copyright 2024 © Institute of Software, CAS. All rights reserved.
// Copyright © 2020, Oracle and/or its affiliates.
// Copyright (c) 2019 Intel Corporation. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//
// Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause

//! Traits and structs for loading pe image kernels into guest memory.

use std::fmt;
use std::io::{Read, Seek, SeekFrom};

use vm_memory::{Address, ByteValued, GuestAddress, GuestMemory, GuestUsize, ReadVolatile};

use crate::loader::{Error as KernelLoaderError, KernelLoader, KernelLoaderResult, Result};

/// ARM64 and RISC-V64 Image (PE) format support
pub struct PE;

#[derive(Debug, PartialEq, Eq)]
/// PE kernel loader errors.
pub enum Error {
    /// Unable to seek to Image end.
    SeekImageEnd,
    /// Unable to seek to Image header.
    SeekImageHeader,
    /// Unable to seek to DTB start.
    SeekDtbStart,
    /// Unable to seek to DTB end.
    SeekDtbEnd,
    /// Device tree binary too big.
    DtbTooBig,
    /// Unable to read kernel image.
    ReadKernelImage,
    /// Unable to read Image header.
    ReadImageHeader,
    /// Unable to read DTB image
    ReadDtbImage,
    /// Invalid Image binary.
    InvalidImage,
    /// Invalid Image magic number.
    InvalidImageMagicNumber,
    /// Invalid base address alignment
    InvalidBaseAddrAlignment,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc = match self {
            Error::SeekImageEnd => "unable to seek Image end",
            Error::SeekImageHeader => "unable to seek Image header",
            Error::ReadImageHeader => "unable to read Image header",
            Error::ReadDtbImage => "unable to read DTB image",
            Error::SeekDtbStart => "unable to seek DTB start",
            Error::SeekDtbEnd => "unable to seek DTB end",
            Error::InvalidImage => "invalid Image",
            Error::InvalidImageMagicNumber => "invalid Image magic number",
            Error::DtbTooBig => "device tree image too big",
            Error::ReadKernelImage => "unable to read kernel image",
            Error::InvalidBaseAddrAlignment => "base address not aligned to 2 MB",
        };

        write!(f, "PE Kernel Loader: {}", desc)
    }
}

impl std::error::Error for Error {}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
// See kernel doc Documentation/arm64/booting.txt for more information.
// All these fields should be little endian.
struct arm64_image_header {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: u64,
    res2: u64,
    res3: u64,
    res4: u64,
    magic: u32,
    res5: u32,
}

#[cfg(target_arch = "aarch64")]
// SAFETY: The layout of the structure is fixed and can be initialized by
// reading its content from byte array.
unsafe impl ByteValued for arm64_image_header {}

#[cfg(target_arch = "riscv64")]
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
// See kernel doc Documentation/arch/riscv/boot-image-header.rst
// All these fields should be little endian.
struct riscv64_image_header {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: u64,
    version: u32,
    res1: u32,
    res2: u64,
    magic: u64,
    magic2: u32,
    res3: u32,
}

#[cfg(target_arch = "riscv64")]
// SAFETY: The layout of the structure is fixed and can be initialized by
// reading its content from byte array.
unsafe impl ByteValued for riscv64_image_header {}

impl KernelLoader for PE {
    /// Loads a PE Image into guest memory.
    ///
    /// # Arguments
    ///
    /// * `guest_mem` - The guest memory where the kernel image is loaded.
    /// * `kernel_offset` - 2MB-aligned base addres in guest memory at which to load the kernel.
    /// * `kernel_image` - Input Image format kernel image.
    /// * `highmem_start_address` - ignored on ARM64 and RISC-V64.
    ///
    /// # Returns
    /// * KernelLoaderResult
    fn load<F, M: GuestMemory>(
        guest_mem: &M,
        kernel_offset: Option<GuestAddress>,
        kernel_image: &mut F,
        _highmem_start_address: Option<GuestAddress>,
    ) -> Result<KernelLoaderResult>
    where
        F: ReadVolatile + Read + Seek,
    {
        let kernel_size = kernel_image
            .seek(SeekFrom::End(0))
            .map_err(|_| Error::SeekImageEnd)? as usize;

        #[cfg(target_arch = "aarch64")]
        let mut image_header: arm64_image_header = Default::default();
        #[cfg(target_arch = "riscv64")]
        let mut image_header: riscv64_image_header = Default::default();

        kernel_image.rewind().map_err(|_| Error::SeekImageHeader)?;

        kernel_image
            .read_exact(image_header.as_mut_slice())
            .map_err(|_| Error::ReadImageHeader)?;

        #[cfg(target_arch = "aarch64")]
        if u32::from_le(image_header.magic) != 0x644d_5241 {
            return Err(Error::InvalidImageMagicNumber.into());
        }
        #[cfg(target_arch = "riscv64")]
        if u32::from_le(image_header.magic2) != 0x0543_5352 {
            return Err(Error::InvalidImageMagicNumber.into());
        }

        #[cfg(target_arch = "aarch64")]
        let text_offset = if u64::from_le(image_header.image_size) == 0 {
            0x80000
        } else {
            u64::from_le(image_header.text_offset)
        };
        #[cfg(target_arch = "riscv64")]
        let text_offset = u64::from_le(image_header.text_offset);

        // Validate that kernel_offset is 2 MB aligned, as required by the
        // arm64 and riscv64 boot protocol
        if let Some(kernel_offset) = kernel_offset {
            if kernel_offset.raw_value() % 0x0020_0000 != 0 {
                return Err(Error::InvalidBaseAddrAlignment.into());
            }
        }

        let mem_offset = kernel_offset
            .unwrap_or(GuestAddress(0))
            .checked_add(text_offset)
            .ok_or(Error::InvalidImage)?;

        let mut loader_result = KernelLoaderResult {
            kernel_load: mem_offset,
            ..Default::default()
        };

        kernel_image.rewind().map_err(|_| Error::SeekImageHeader)?;
        guest_mem
            .read_exact_volatile_from(mem_offset, kernel_image, kernel_size)
            .map_err(|_| Error::ReadKernelImage)?;

        loader_result.kernel_end = mem_offset
            .raw_value()
            .checked_add(kernel_size as GuestUsize)
            .ok_or(KernelLoaderError::MemoryOverflow)?;

        Ok(loader_result)
    }
}

/// Writes the device tree to the given memory slice.
///
/// # Arguments
///
/// * `guest_mem` - A u8 slice that will be partially overwritten by the device tree blob.
/// * `guest_addr` - The address in `guest_mem` at which to load the device tree blob.
/// * `dtb_image` - The device tree blob.
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub fn load_dtb<F, M: GuestMemory>(
    guest_mem: &M,
    guest_addr: GuestAddress,
    dtb_image: &mut F,
) -> Result<()>
where
    F: ReadVolatile + Read + Seek,
{
    let dtb_size = dtb_image
        .seek(SeekFrom::End(0))
        .map_err(|_| Error::SeekDtbEnd)? as usize;
    if dtb_size > 0x200000 {
        return Err(Error::DtbTooBig.into());
    }
    dtb_image.rewind().map_err(|_| Error::SeekDtbStart)?;
    guest_mem
        .read_exact_volatile_from(guest_addr, dtb_image, dtb_size)
        .map_err(|_| Error::ReadDtbImage.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use vm_memory::{Address, GuestAddress};
    type GuestMemoryMmap = vm_memory::GuestMemoryMmap<()>;

    const MEM_SIZE: u64 = 0x100_0000;

    fn create_guest_mem() -> GuestMemoryMmap {
        GuestMemoryMmap::from_ranges(&[(GuestAddress(0x0), (MEM_SIZE as usize))]).unwrap()
    }

    fn make_image_bin() -> Vec<u8> {
        let mut v = Vec::new();
        #[cfg(target_arch = "aarch64")]
        v.extend_from_slice(include_bytes!("test_arm64_image.bin"));
        #[cfg(target_arch = "riscv64")]
        v.extend_from_slice(include_bytes!("test_riscv64_image.bin"));
        v
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn load_arm64_image() {
        let gm = create_guest_mem();
        let mut image = make_image_bin();
        let kernel_addr = GuestAddress(0x200000);

        let loader_result =
            PE::load(&gm, Some(kernel_addr), &mut Cursor::new(&image), None).unwrap();
        assert_eq!(loader_result.kernel_load.raw_value(), 0x280000);
        assert_eq!(loader_result.kernel_end, 0x281000);

        // Attempt to load the kernel at an address that is not aligned to 2MB boundary
        let kernel_offset = GuestAddress(0x0030_0000);
        let loader_result = PE::load(&gm, Some(kernel_offset), &mut Cursor::new(&image), None);
        assert_eq!(
            loader_result,
            Err(KernelLoaderError::Pe(Error::InvalidBaseAddrAlignment))
        );

        image[0x39] = 0x0;
        let loader_result = PE::load(&gm, Some(kernel_addr), &mut Cursor::new(&image), None);
        assert_eq!(
            loader_result,
            Err(KernelLoaderError::Pe(Error::InvalidImageMagicNumber))
        );
    }

    #[cfg(target_arch = "riscv64")]
    #[test]
    fn load_riscv64_image() {
        let gm = create_guest_mem();
        let mut image = make_image_bin();
        let kernel_addr = GuestAddress(0x400000);

        let loader_result =
            PE::load(&gm, Some(kernel_addr), &mut Cursor::new(&image), None).unwrap();
        assert_eq!(loader_result.kernel_load.raw_value(), 0x600000);
        assert_eq!(loader_result.kernel_end, 0x601000);

        // Attempt to load the kernel at an address that is not aligned to 2MB boundary
        let kernel_offset = GuestAddress(0x0030_0000);
        let loader_result = PE::load(&gm, Some(kernel_offset), &mut Cursor::new(&image), None);
        assert_eq!(
            loader_result,
            Err(KernelLoaderError::Pe(Error::InvalidBaseAddrAlignment))
        );

        image[0x38] = 0x0;
        let loader_result = PE::load(&gm, Some(kernel_addr), &mut Cursor::new(&image), None);
        assert_eq!(
            loader_result,
            Err(KernelLoaderError::Pe(Error::InvalidImageMagicNumber))
        );
    }
}