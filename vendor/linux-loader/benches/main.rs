// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-BSD-3-Clause file.
//
// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause

extern crate criterion;
extern crate linux_loader;
extern crate vm_memory;

use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use x86_64::*;

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
mod fdt;
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub use fdt::*;

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(500);
    targets = criterion_benchmark
}

criterion_main! {
    benches
}