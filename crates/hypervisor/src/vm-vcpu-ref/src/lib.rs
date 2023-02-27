// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![deny(missing_docs)]
//! The `vm-vcpu-ref` crate provides abstractions for setting up the `VM` and `vCPUs` for booting.
//! The high level interface exported by this crate is uniform on both supported platforms
//! (x86_64 and aarch64). Differences only arise in configuration parameters as there are
//! features only supported on one platform (i.e. CPUID on x86_64), and in the saved/restored
//! state as both platforms define registers and VM/vCPU specific features differently.

/// Helpers for setting up the `VM` for running on x86_64.
pub mod x86_64;

/// Helpers for setting up the `VM` for running on aarch64.
pub mod aarch64;
