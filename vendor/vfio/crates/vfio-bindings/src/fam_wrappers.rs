// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//

use crate::bindings::vfio::*;
use vmm_sys_util::fam::{FamStruct, FamStructWrapper};

const MSIX_MAX_VECTORS: usize = 2048;

// Implement the FamStruct trait vfio_irq_set.
generate_fam_struct_impl!(vfio_irq_set, u8, data, u32, count, MSIX_MAX_VECTORS);

/// Wrapper over the `vfio_irq_set` structure.
///
/// The `vfio_irq_set` structure contains a flexible array member. For details check the
/// [VFIO userspace API](https://github.com/torvalds/linux/blob/master/include/uapi/linux/vfio.h)
/// documentation on `vfio_irq_set`. To provide safe access to the array
/// elements, this type is implemented using
/// [FamStructWrapper](../vmm_sys_util/fam/struct.FamStructWrapper.html).
pub type IrqSet = FamStructWrapper<vfio_irq_set>;

#[cfg(test)]
mod tests {
    extern crate byteorder;

    use super::*;
    use byteorder::{ByteOrder, LittleEndian};
    use std::mem;

    fn vec_with_size_in_bytes<T: Default>(size_in_bytes: usize) -> Vec<T> {
        let rounded_size = (size_in_bytes + mem::size_of::<T>() - 1) / mem::size_of::<T>();
        let mut v = Vec::with_capacity(rounded_size);
        for _ in 0..rounded_size {
            v.push(T::default())
        }
        v
    }

    fn vec_with_array_field<T: Default, F>(count: usize) -> Vec<T> {
        let element_space = count * mem::size_of::<F>();
        let vec_size_bytes = mem::size_of::<T>() + element_space;
        vec_with_size_in_bytes(vec_size_bytes)
    }

    // Opinionated PartialEq implementation for vfio_irq_set.
    impl PartialEq for vfio_irq_set {
        fn eq(&self, other: &Self) -> bool {
            if self.argsz != other.argsz
                || self.flags != other.flags
                || self.index != other.index
                || self.start != other.start
                || self.count != other.count
            {
                return false;
            }
            true
        }
    }

    #[test]
    fn irqset_fam_test() {
        let event_fds: Vec<u32> = vec![0, 1, 2, 3, 4, 5];

        // Build a FAM wrapper for this vfio_irq_set.
        let mut irq_set_wrapper = IrqSet::new(event_fds.len() * mem::size_of::<u32>()).unwrap();
        // SAFETY: Safe as we create the irq_set_wrapper with the constructor
        let irq_set_fam = unsafe { irq_set_wrapper.as_mut_fam_struct() };

        let fds_fam = irq_set_fam.as_mut_slice();
        for (index, event_fd) in event_fds.iter().enumerate() {
            let fds_offset = index * mem::size_of::<u32>();
            let fd = &mut fds_fam[fds_offset..fds_offset + mem::size_of::<u32>()];
            LittleEndian::write_u32(fd, *event_fd);
        }

        irq_set_fam.argsz = mem::size_of::<vfio_irq_set>() as u32
            + (event_fds.len() * mem::size_of::<u32>()) as u32;
        irq_set_fam.flags = VFIO_IRQ_SET_DATA_EVENTFD | VFIO_IRQ_SET_ACTION_TRIGGER;
        irq_set_fam.index = 1;
        irq_set_fam.start = 0;
        irq_set_fam.count = event_fds.len() as u32;

        // Build the same vfio_irq_set structure with the vec_with_array routines
        let mut irq_set_vec = vec_with_array_field::<vfio_irq_set, u32>(event_fds.len());
        irq_set_vec[0].argsz = mem::size_of::<vfio_irq_set>() as u32
            + (event_fds.len() * mem::size_of::<u32>()) as u32;
        irq_set_vec[0].flags = VFIO_IRQ_SET_DATA_EVENTFD | VFIO_IRQ_SET_ACTION_TRIGGER;
        irq_set_vec[0].index = 1;
        irq_set_vec[0].start = 0;
        irq_set_vec[0].count = event_fds.len() as u32;

        // SAFETY: irq_set_vec is a valid flexible array constructed by us.
        let fds_vec = unsafe {
            irq_set_vec[0]
                .data
                .as_mut_slice(event_fds.len() * mem::size_of::<u32>())
        };
        for (index, event_fd) in event_fds.iter().enumerate() {
            let fds_offset = index * mem::size_of::<u32>();
            let fd = &mut fds_vec[fds_offset..fds_offset + mem::size_of::<u32>()];
            LittleEndian::write_u32(fd, *event_fd);
        }

        // Both sets should be identical.
        assert_eq!(
            irq_set_vec
                .iter()
                .zip(irq_set_wrapper.into_raw().iter())
                .filter(|&(a, b)| a == b)
                .count(),
            irq_set_vec.len()
        );
    }
}