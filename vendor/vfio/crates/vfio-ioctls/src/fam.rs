// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// This is a private version of vmm-sys-util::FamStruct. As it works smoothly, we keep it for
// simplicity.

use std::mem::size_of;

/// Returns a `Vec<T>` with a size in bytes at least as large as `size_in_bytes`.
fn vec_with_size_in_bytes<T: Default>(size_in_bytes: usize) -> Vec<T> {
    let rounded_size = size_in_bytes.div_ceil(size_of::<T>());
    let mut v = Vec::with_capacity(rounded_size);
    for _ in 0..rounded_size {
        v.push(T::default())
    }
    v
}

/// The VFIO API has several structs that resembles the following `Foo` structure:
///
/// ```
/// struct ControlMessageHeader {
///     r#type: u8,
///     length: u8,
/// }
///
/// #[repr(C)]
/// pub struct __IncompleteArrayField<T>(::std::marker::PhantomData<T>);
/// #[repr(C)]
/// struct Foo {
///     some_data: ControlMessageHeader,
///     entries: __IncompleteArrayField<u32>,
/// }
/// ```
///
/// In order to allocate such a structure, `size_of::<Foo>()` would be too small because it would not
/// include any space for `entries`. To make the allocation large enough while still being aligned
/// for `Foo`, a `Vec<Foo>` is created. Only the first element of `Vec<Foo>` would actually be used
/// as a `Foo`. The remaining memory in the `Vec<Foo>` is for `entries`, which must be contiguous
/// with `Foo`. This function is used to make the `Vec<Foo>` with enough space for `count` entries.
pub(crate) fn vec_with_array_field<T: Default, F>(count: usize) -> Vec<T> {
    let element_space = match count.checked_mul(size_of::<F>()) {
        None => panic!("allocating too large buffer with vec_with_array_field"),
        Some(v) => v,
    };
    let vec_size_bytes = match element_space.checked_add(size_of::<T>()) {
        None => panic!("allocating too large buffer with vec_with_array_field"),
        Some(v) => v,
    };

    vec_with_size_in_bytes(vec_size_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    #[allow(dead_code)]
    struct Header {
        ty: u32,
        len: u32,
    }

    #[allow(dead_code)]
    struct Field {
        f1: u64,
        f2: u64,
    }

    #[test]
    fn test_vec_with_array_field() {
        let v1 = vec_with_array_field::<Header, Field>(1);
        assert_eq!(v1.len(), 3);

        let v2 = vec_with_array_field::<Header, Field>(0);
        assert_eq!(v2.len(), 1);

        let v3 = vec_with_array_field::<Header, Field>(5);
        assert_eq!(v3.len(), 11);
    }

    #[test]
    #[should_panic]
    fn test_vec_with_array_field_overflow() {
        let _ = vec_with_array_field::<Header, Field>(usize::MAX);
    }
}