// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: (BSD-3-Clause OR Apache-2.0)

#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Keep this until https://github.com/rust-lang/rust-bindgen/issues/1651 is fixed.
#![cfg_attr(test, allow(deref_nullptr))]

#[allow(clippy::undocumented_unsafe_blocks)]
pub mod vfio;