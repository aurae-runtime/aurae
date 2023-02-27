// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

// This crate holds devices used by the VMM. They are reusing rust-vmm generic components, and
// we are striving to turn as much of the local code as possible into reusable building blocks
// going forward.

pub mod legacy;
pub mod virtio;
