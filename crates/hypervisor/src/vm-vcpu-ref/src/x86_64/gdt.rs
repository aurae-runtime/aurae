// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions Copyright 2017 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the THIRD-PARTY file.

//! The `gdt` module provides abstractions for building a Global Descriptor Table (GDT).
//!
//! These abstractions are built to resemble the structures defined in the
//! [Intel Manual](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf).

use kvm_bindings::kvm_segment;
use vm_memory::{ByteValued, Bytes, GuestAddress, GuestMemory, GuestMemoryError};

use std::mem;

/// The offset at which GDT resides in memory.
pub const BOOT_GDT_OFFSET: u64 = 0x500;
/// The offset at which IDT resides in memory.
pub const BOOT_IDT_OFFSET: u64 = 0x520;
/// Maximum number of GDT entries as defined in the Intel Specification.
pub const MAX_GDT_SIZE: usize = 1 << 13;

/// Errors associated with operations on the GDT.
#[derive(Debug)]
pub enum Error {
    /// Invalid memory access.
    GuestMemory(GuestMemoryError),
    /// Too many entries in the GDT.
    TooManyEntries,
}

impl From<GuestMemoryError> for Error {
    fn from(inner: GuestMemoryError) -> Self {
        Error::GuestMemory(inner)
    }
}

/// Results corresponding to operations on the GDT.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Default, Debug)]
/// A segment descriptor is a data structure in a GDT (Global Descriptor Table)
/// that provides the processor with the size and location of a segment, as
/// well as access control and status information.
///
/// The segment descriptor is referred to in the Linux Kernel as a GDT entry.
/// More information about the segment descriptor, its inner structure, and
/// types of segment descriptors is available in Section "3.4.5 Segment
/// Descriptors" of the Intel Manual Volume 3a.
pub struct SegmentDescriptor(pub u64);

// Safe because SegmentDescriptor is just a wrapper over u64.
unsafe impl ByteValued for SegmentDescriptor {}

impl SegmentDescriptor {
    /// Compute the segment descriptor (also called GDT entry in the Linux Kernel)
    /// based on base, flags, and limit.
    ///
    /// The segment descriptor can then be used to create the Global Descriptor
    /// Table (GDT). For more details, check out the [Gdt constructor](Gdt::new).
    ///
    /// # Arguments
    /// * `base`: A 32-bit value containing the linear address where the
    ///           segment begins.
    /// * `limit`: A 20-bit value (the most significant 12 bits are ignored from
    ///            the value) that tells the maximum addressable unit.
    /// * `flags`: Depending on the segment type, the flags set up various
    ///            properties such as the privilege level, type, and granularity.
    ///            The full list of available flags is available in section
    ///            3.4.5 of the Intel® 64 and IA-32 Architectures Developer's
    ///            Manual: Vol. 3A.
    pub fn from(flags: u16, base: u32, limit: u32) -> Self {
        // The segment descriptor has the following inner structure:
        // |31                                    16|15                               0|
        // |          Base 0:15                     |          Limit 0:15              |
        // |63 - - - - - - 56 55 - - 52 51 - -    48 47 - - - - - - 40 39 - - - - - - 32
        // |Base 24:31       | Flags   |Limit 16:19 |  Access Bytes   | Base 16:23     |
        // A more in depth description is available:
        // https://wiki.osdev.org/Global_Descriptor_Table.
        //
        // The code below is adapted from the Linux kernel, and it can be found in
        // the arch/x86/include/asm/segment.h file.
        //
        // The way flags work is a bit complicated because Access Bytes are part of
        // flags, then there is the Limit (which we need to ignore when setting the
        // flags, then there are more flags (from bits 52 to 55)).
        // It might make sense in the future to create a wrapper over the flags
        // so that it's easier to set it; for example, we can have something like:
        // Flags::set_available(val).set_present(..) and so on. The flags are
        // defined in Figure 3-8. Segment Descriptor table of the intel manual.
        // The getters below (like avl, g, l, p) also are for the flags and their offset
        // is defined in the spec.
        SegmentDescriptor(
            ((u64::from(base) & 0xff00_0000u64) << (56 - 24))
                | ((u64::from(flags) & 0x0000_f0ffu64) << 40)
                | ((u64::from(limit) & 0x000f_0000u64) << (48 - 16))
                | ((u64::from(base) & 0x00ff_ffffu64) << 16)
                | (u64::from(limit) & 0x0000_ffffu64),
        )
    }

    fn base(&self) -> u64 {
        (((self.0) & 0xff00_0000_0000_0000) >> 32)
            | (((self.0) & 0x0000_00ff_0000_0000) >> 16)
            | (((self.0) & 0x0000_0000_ffff_0000) >> 16)
    }

    fn limit(&self) -> u32 {
        ((((self.0) & 0x000f_0000_0000_0000) >> 32) | ((self.0) & 0x0000_0000_0000_ffff)) as u32
    }

    fn g(&self) -> u8 {
        ((self.0 & 0x0080_0000_0000_0000) >> 55) as u8
    }

    fn db(&self) -> u8 {
        ((self.0 & 0x0040_0000_0000_0000) >> 54) as u8
    }

    fn l(&self) -> u8 {
        ((self.0 & 0x0020_0000_0000_0000) >> 53) as u8
    }

    fn avl(&self) -> u8 {
        ((self.0 & 0x0010_0000_0000_0000) >> 52) as u8
    }

    fn p(&self) -> u8 {
        ((self.0 & 0x0000_8000_0000_0000) >> 47) as u8
    }

    fn dpl(&self) -> u8 {
        ((self.0 & 0x0000_6000_0000_0000) >> 45) as u8
    }

    fn s(&self) -> u8 {
        ((self.0 & 0x0000_1000_0000_0000) >> 44) as u8
    }

    fn segment_type(&self) -> u8 {
        ((self.0 & 0x0000_0f00_0000_0000) >> 40) as u8
    }

    /// Build a `kvm_segment` from a GDT segment descriptor and the
    /// selector (`table_index`).
    ///
    /// # Arguments
    ///
    /// * `table_index` - Index of the entry in the gdt table.
    fn create_kvm_segment(&self, table_index: usize) -> kvm_segment {
        kvm_segment {
            base: self.base(),
            limit: self.limit(),
            // The multiplication is safe because the table_index can be maximum
            // `MAX_GDT_SIZE`. The conversion is safe because the result fits in u16.
            selector: (table_index * 8) as u16,
            type_: self.segment_type(),
            present: self.p(),
            dpl: self.dpl(),
            db: self.db(),
            s: self.s(),
            l: self.l(),
            g: self.g(),
            avl: self.avl(),
            padding: 0,
            unusable: match self.p() {
                0 => 1,
                _ => 0,
            },
        }
    }
}

#[derive(Clone, Debug)]
/// The `Gdt` is a wrapper for creating and managing operations on the
/// Global Descriptor Table (GDT). The GDT is a data structure used by
/// Intel x86-family processors to define the characteristics of the
/// various memory areas used during program execution (like data, code, TSS).
///
/// The `Gdt` provides a default implementation that can be used for setting
/// up a vCPU for booting. The default implementation contains all 4 segment
/// descriptors corresponding to null, code, data, and TSS. The default can be
/// extended by using `push`. When the default is not matching the product
/// requirements, the GDT can also be created from scratch.
pub struct Gdt(Vec<SegmentDescriptor>);

impl Gdt {
    /// Create an empty `GDT`.
    ///
    /// # Example - Creating a GDT from scratch
    ///
    /// ```rust
    /// use vm_vcpu_ref::x86_64::gdt::{Gdt, SegmentDescriptor};
    ///
    /// let mut gdt = Gdt::new();
    /// // Create the GDT as in the GDT Tutorial: https://wiki.osdev.org/GDT_Tutorial
    /// // The following unwraps are safe because we're adding less than `MAX_GDT_SIZE` elements.
    /// gdt.try_push(SegmentDescriptor::from(0, 0, 0)).unwrap();
    /// gdt.try_push(SegmentDescriptor::from(0x9A, 0, 0xffffffff))
    ///     .unwrap();
    /// gdt.try_push(SegmentDescriptor::from(0x92, 0, 0xffffffff))
    ///     .unwrap();
    /// gdt.try_push(SegmentDescriptor::from(0x89, 0, 0xffffffff))
    ///     .unwrap();
    /// ```
    pub fn new() -> Gdt {
        Gdt(vec![])
    }

    /// Try to push `entry` into the `Gdt`.
    ///
    /// Returns an error when there is no more space available.
    pub fn try_push(&mut self, entry: SegmentDescriptor) -> Result<()> {
        if self.0.len() >= MAX_GDT_SIZE {
            return Err(Error::TooManyEntries);
        }
        self.0.push(entry);
        Ok(())
    }

    /// Create a KVM segment from the GDT entry available at `index`.
    pub fn create_kvm_segment_for(&self, index: usize) -> Option<kvm_segment> {
        self.0
            .get(index)
            .map(|entry| entry.create_kvm_segment(index))
    }

    /// Writes the GDT (Global Descriptor Table) into Guest Memory.
    ///
    /// # Example - Creating the GDT for Linux
    ///
    /// ```rust
    /// use vm_memory::{GuestAddress, GuestMemoryMmap};
    /// use vm_vcpu_ref::x86_64::gdt::Gdt;
    ///
    /// let guest_memory: GuestMemoryMmap =
    ///     GuestMemoryMmap::from_ranges(&[(GuestAddress(0), 1024 << 20)]).unwrap();
    /// let gdt_table = Gdt::default();
    /// gdt_table.write_to_mem(&guest_memory).unwrap();
    /// ```
    pub fn write_to_mem<Memory: GuestMemory>(&self, mem: &Memory) -> Result<()> {
        let boot_gdt_addr = GuestAddress(BOOT_GDT_OFFSET);
        for (index, entry) in self.0.iter().enumerate() {
            // The multiplication below cannot fail because we can have maximum 8192 entries in
            // the gdt table, and 8192 * 4 (size_of::<u64>) fits in usize
            let addr = mem
                .checked_offset(boot_gdt_addr, index * mem::size_of::<SegmentDescriptor>())
                .ok_or(GuestMemoryError::InvalidGuestAddress(boot_gdt_addr))?;
            mem.write_obj(*entry, addr)?;
        }
        Ok(())
    }
}

impl Default for Gdt {
    fn default() -> Self {
        Gdt(vec![
            SegmentDescriptor::from(0, 0, 0),            // NULL
            SegmentDescriptor::from(0xa09b, 0, 0xfffff), // CODE
            SegmentDescriptor::from(0xc093, 0, 0xfffff), // DATA
            SegmentDescriptor::from(0x808b, 0, 0xfffff), // TSS
        ])
    }
}

/// Write `val` in Guest Memory at the address corresponding to the
/// IDT offset.
pub fn write_idt_value<Memory: GuestMemory>(val: u64, guest_mem: &Memory) -> Result<()> {
    let boot_idt_addr = GuestAddress(BOOT_IDT_OFFSET);
    guest_mem
        .write_obj(val, boot_idt_addr)
        .map_err(Error::GuestMemory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm_memory::GuestMemoryMmap;

    #[test]
    fn test_kvm_segment_parse() {
        let desc = SegmentDescriptor::from(0xA09B, 0x10_0000, 0xfffff);
        let mut gdt = Gdt::new();
        gdt.try_push(desc).unwrap();

        let seg = gdt.create_kvm_segment_for(0).unwrap();
        // 0xA09B
        // 'A'
        assert_eq!(0x1, seg.g);
        assert_eq!(0x0, seg.db);
        assert_eq!(0x1, seg.l);
        assert_eq!(0x0, seg.avl);
        // '9'
        assert_eq!(0x1, seg.present);
        assert_eq!(0x0, seg.dpl);
        assert_eq!(0x1, seg.s);
        // 'B'
        assert_eq!(0xB, seg.type_);
        // base and limit
        assert_eq!(0x10_0000, seg.base);
        assert_eq!(0xfffff, seg.limit);
        assert_eq!(0x0, seg.unusable);

        // Trying to fetch an invalid index returns none.
        assert_eq!(gdt.create_kvm_segment_for(1), None);
        assert_eq!(gdt.create_kvm_segment_for(MAX_GDT_SIZE + 1), None);
    }

    #[test]
    fn test_write_table() {
        // Error case: create a guest memory with a size smaller then the address where the GDT
        // is written.
        let gm_size = BOOT_GDT_OFFSET - 100;
        let guest_memory: GuestMemoryMmap =
            GuestMemoryMmap::from_ranges(&[(GuestAddress(0), gm_size as usize)]).unwrap();

        let mut gdt_table = Gdt::new();
        gdt_table
            .try_push(SegmentDescriptor::from(0xA09B, 0x10_0000, 0xfffff))
            .unwrap();

        let err = gdt_table.write_to_mem(&guest_memory).unwrap_err();
        assert!(format!("{:#?}", err).contains("InvalidGuestAddress"));

        // Error case: writing the IDT also returns an error as memory is too small.
        let err = write_idt_value(0, &guest_memory).unwrap_err();
        assert!(format!("{:#?}", err).contains("InvalidGuestAddress"));

        // Writing the default GDT and IDT in a normal sized memory works.
        let guest_memory: GuestMemoryMmap =
            GuestMemoryMmap::from_ranges(&[(GuestAddress(0), 1024 << 20)]).unwrap();
        let gdt_table = Gdt::default();
        assert_eq!(gdt_table.0.len(), 4);

        assert!(gdt_table.write_to_mem(&guest_memory).is_ok());
        assert!(write_idt_value(0, &guest_memory).is_ok());
    }

    #[test]
    fn test_too_many_entries() {
        let mut gdt = Gdt::new();
        // Pushing maximum allowed number of entries works.
        for i in 0..MAX_GDT_SIZE {
            gdt.try_push(SegmentDescriptor(i as u64)).unwrap();
        }

        assert_eq!(gdt.0.len(), MAX_GDT_SIZE);
        // Pushing one more element returns an error.
        assert!(gdt.try_push(SegmentDescriptor(0)).is_err());
    }

    #[test]
    fn test_memory_constraints() {
        // The Segment descriptor needs to fit in an u64 otherwise the offsets in
        // the GDT are going to be wrong.
        assert_eq!(mem::size_of::<u64>(), mem::size_of::<SegmentDescriptor>());
    }
}
