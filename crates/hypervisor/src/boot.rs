// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
#![cfg(target_arch = "x86_64")]
use std::result;

use linux_loader::{bootparam::boot_params, loader::KernelLoaderResult};
use vm_memory::{Address, GuestAddress, GuestMemory, GuestMemoryMmap};

// x86_64 boot constants. See https://www.kernel.org/doc/Documentation/x86/boot.txt for the full
// documentation.
// Header field: `boot_flag`. Must contain 0xaa55. This is the closest thing old Linux kernels
// have to a magic number.
const KERNEL_BOOT_FLAG_MAGIC: u16 = 0xaa55;
// Header field: `header`. Must contain the magic number `HdrS` (0x5372_6448).
const KERNEL_HDR_MAGIC: u32 = 0x5372_6448;
// Header field: `type_of_loader`. Unless using a pre-registered bootloader (which we aren't), this
// field must be set to 0xff.
const KERNEL_LOADER_OTHER: u8 = 0xff;
// Header field: `kernel_alignment`. Alignment unit required by a relocatable kernel.
const KERNEL_MIN_ALIGNMENT_BYTES: u32 = 0x0100_0000;

// Start address for the EBDA (Extended Bios Data Area). Older computers (like the one this VMM
// emulates) typically use 1 KiB for the EBDA, starting at 0x9fc00.
// See https://wiki.osdev.org/Memory_Map_(x86) for more information.
const EBDA_START: u64 = 0x0009_fc00;
// RAM memory type.
// TODO: this should be bindgen'ed and exported by linux-loader.
// See https://github.com/rust-vmm/linux-loader/issues/51
const E820_RAM: u32 = 1;

#[derive(Debug, PartialEq, Eq)]
/// Errors pertaining to boot parameter setup.
pub enum Error {
    /// Invalid E820 configuration.
    E820Configuration,
    /// Highmem start address is past the guest memory end.
    HimemStartPastMemEnd,
    /// Highmem start address is past the MMIO gap start.
    HimemStartPastMmioGapStart,
    /// The MMIO gap end is past the guest memory end.
    MmioGapPastMemEnd,
    /// The MMIO gap start is past the gap end.
    MmioGapStartPastMmioGapEnd,
}

fn add_e820_entry(
    params: &mut boot_params,
    addr: u64,
    size: u64,
    mem_type: u32,
) -> result::Result<(), Error> {
    if params.e820_entries >= params.e820_table.len() as u8 {
        return Err(Error::E820Configuration);
    }

    params.e820_table[params.e820_entries as usize].addr = addr;
    params.e820_table[params.e820_entries as usize].size = size;
    params.e820_table[params.e820_entries as usize].type_ = mem_type;
    params.e820_entries += 1;

    Ok(())
}

/// Build boot parameters for ELF kernels following the Linux boot protocol.
///
/// # Arguments
///
/// * `guest_memory` - guest memory.
/// * `kernel_load` - result of loading the kernel in guest memory.
/// * `himem_start` - address where high memory starts.
/// * `mmio_gap_start` - address where the MMIO gap starts.
/// * `mmio_gap_end` - address where the MMIO gap ends.
pub fn build_bootparams(
    guest_memory: &GuestMemoryMmap,
    kernel_load: &KernelLoaderResult,
    himem_start: GuestAddress,
    mmio_gap_start: GuestAddress,
    mmio_gap_end: GuestAddress,
) -> result::Result<boot_params, Error> {
    if mmio_gap_start >= mmio_gap_end {
        return Err(Error::MmioGapStartPastMmioGapEnd);
    }

    let mut params = boot_params::default();

    if let Some(hdr) = kernel_load.setup_header {
        params.hdr = hdr;
    } else {
        params.hdr.boot_flag = KERNEL_BOOT_FLAG_MAGIC;
        params.hdr.header = KERNEL_HDR_MAGIC;
        params.hdr.kernel_alignment = KERNEL_MIN_ALIGNMENT_BYTES;
    }
    // If the header copied from the bzImage file didn't set type_of_loader,
    // force it to "undefined" so that the guest can boot normally.
    // See: https://github.com/cloud-hypervisor/cloud-hypervisor/issues/918
    // and: https://www.kernel.org/doc/html/latest/x86/boot.html#details-of-header-fields
    if params.hdr.type_of_loader == 0 {
        params.hdr.type_of_loader = KERNEL_LOADER_OTHER;
    }

    // Add an entry for EBDA itself.
    add_e820_entry(&mut params, 0, EBDA_START, E820_RAM)?;

    // Add entries for the usable RAM regions (potentially surrounding the MMIO gap).
    let last_addr = guest_memory.last_addr();
    if last_addr < mmio_gap_start {
        add_e820_entry(
            &mut params,
            himem_start.raw_value(),
            // The unchecked + 1 is safe because:
            // * overflow could only occur if last_addr - himem_start == u64::MAX
            // * last_addr is smaller than mmio_gap_start, a valid u64 value
            // * last_addr - himem_start is also smaller than mmio_gap_start
            last_addr
                .checked_offset_from(himem_start)
                .ok_or(Error::HimemStartPastMemEnd)?
                + 1,
            E820_RAM,
        )?;
    } else {
        add_e820_entry(
            &mut params,
            himem_start.raw_value(),
            mmio_gap_start
                .checked_offset_from(himem_start)
                .ok_or(Error::HimemStartPastMmioGapStart)?,
            E820_RAM,
        )?;

        if last_addr > mmio_gap_end {
            add_e820_entry(
                &mut params,
                mmio_gap_end.raw_value(),
                // The unchecked_offset_from is safe, guaranteed by the `if` condition above.
                // The unchecked + 1 is safe because:
                // * overflow could only occur if last_addr == u64::MAX and mmio_gap_end == 0
                // * mmio_gap_end > mmio_gap_start, which is a valid u64 => mmio_gap_end > 0
                last_addr.unchecked_offset_from(mmio_gap_end) + 1,
                E820_RAM,
            )?;
        }
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{DEFAULT_HIGH_RAM_START, MMIO_GAP_END, MMIO_GAP_START};
    use linux_loader::bootparam;

    #[test]
    fn test_build_bootparams() {
        let guest_memory = GuestMemoryMmap::default();
        let mut kern_load_res = KernelLoaderResult::default();

        // Error case: MMIO gap start address is past its end address.
        assert_eq!(
            build_bootparams(
                &guest_memory,
                &kern_load_res,
                GuestAddress(DEFAULT_HIGH_RAM_START),
                GuestAddress(MMIO_GAP_START),
                GuestAddress(MMIO_GAP_START - 1)
            )
            .err(),
            Some(Error::MmioGapStartPastMmioGapEnd)
        );

        // Error case: high memory starts after guest memory ends.
        let guest_memory =
            GuestMemoryMmap::from_ranges(&[(GuestAddress(0), DEFAULT_HIGH_RAM_START as usize - 1)])
                .unwrap();
        assert_eq!(
            build_bootparams(
                &guest_memory,
                &kern_load_res,
                GuestAddress(DEFAULT_HIGH_RAM_START),
                GuestAddress(MMIO_GAP_START),
                GuestAddress(MMIO_GAP_END)
            )
            .err(),
            Some(Error::HimemStartPastMemEnd)
        );

        // Error case: MMIO gap starts before high memory.
        let guest_memory = GuestMemoryMmap::from_ranges(&[
            (GuestAddress(0), MMIO_GAP_START as usize),
            (GuestAddress(MMIO_GAP_END), 0x1000),
        ])
        .unwrap();
        assert_eq!(
            build_bootparams(
                &guest_memory,
                &kern_load_res,
                GuestAddress(MMIO_GAP_START + 1),
                GuestAddress(MMIO_GAP_START),
                GuestAddress(MMIO_GAP_END)
            )
            .err(),
            Some(Error::HimemStartPastMmioGapStart)
        );

        // Success case: 2 ranges surrounding the MMIO gap.
        // Setup header is specified in the kernel loader result.
        kern_load_res.setup_header = Some(bootparam::setup_header::default());
        let params = build_bootparams(
            &guest_memory,
            &kern_load_res,
            GuestAddress(DEFAULT_HIGH_RAM_START),
            GuestAddress(MMIO_GAP_START),
            GuestAddress(MMIO_GAP_END),
        )
        .unwrap();

        // The kernel loader type should have been modified in the setup header.
        let expected_setup_hdr = bootparam::setup_header {
            type_of_loader: KERNEL_LOADER_OTHER,
            ..Default::default()
        };
        assert_eq!(expected_setup_hdr, params.hdr);

        // There should be 3 EBDA entries: EBDA, RAM preceding MMIO gap, RAM succeeding MMIO gap.
        assert_eq!(params.e820_entries, 3);

        // Success case: 1 range preceding the MMIO gap.
        // Let's skip the setup header this time.
        let guest_memory =
            GuestMemoryMmap::from_ranges(&[(GuestAddress(0), MMIO_GAP_START as usize)]).unwrap();
        kern_load_res.setup_header = None;
        let params = build_bootparams(
            &guest_memory,
            &kern_load_res,
            GuestAddress(DEFAULT_HIGH_RAM_START),
            GuestAddress(MMIO_GAP_START),
            GuestAddress(MMIO_GAP_END),
        )
        .unwrap();

        // The setup header should be filled in, even though we didn't specify one.
        let expected_setup_hdr = bootparam::setup_header {
            boot_flag: KERNEL_BOOT_FLAG_MAGIC,
            header: KERNEL_HDR_MAGIC,
            kernel_alignment: KERNEL_MIN_ALIGNMENT_BYTES,
            type_of_loader: KERNEL_LOADER_OTHER,
            ..Default::default()
        };
        assert_eq!(expected_setup_hdr, params.hdr);

        // There should be 2 EBDA entries: EBDA and RAM.
        assert_eq!(params.e820_entries, 2);
    }

    #[test]
    fn test_add_e820_entry() {
        let mut params = boot_params::default();
        assert!(add_e820_entry(&mut params, 0, 0, 0).is_ok());
        params.e820_entries = params.e820_table.len() as u8;
        assert_eq!(
            add_e820_entry(&mut params, 0, 0, 0).err(),
            Some(Error::E820Configuration)
        );
    }
}
