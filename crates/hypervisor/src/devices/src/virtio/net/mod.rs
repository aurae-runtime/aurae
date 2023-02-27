// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

mod bindings;
mod device;
mod queue_handler;
mod simple_handler;
pub mod tap;

pub use device::Net;

// TODO: Move relevant defines to vm-virtio crate.

// Values taken from the virtio standard (section 5.1.3 of the 1.1 version).
pub mod features {
    pub const VIRTIO_NET_F_CSUM: u64 = 0;
    pub const VIRTIO_NET_F_GUEST_CSUM: u64 = 1;
    pub const VIRTIO_NET_F_GUEST_TSO4: u64 = 7;
    pub const VIRTIO_NET_F_GUEST_TSO6: u64 = 8;
    pub const VIRTIO_NET_F_GUEST_UFO: u64 = 10;
    pub const VIRTIO_NET_F_HOST_TSO4: u64 = 11;
    pub const VIRTIO_NET_F_HOST_TSO6: u64 = 12;
    pub const VIRTIO_NET_F_HOST_UFO: u64 = 14;
}

// Size of the `virtio_net_hdr` structure defined by the standard.
pub const VIRTIO_NET_HDR_SIZE: usize = 12;

// Net device ID as defined by the standard.
pub const NET_DEVICE_ID: u32 = 1;

// Prob have to find better names here, but these basically represent the order of the queues.
// If the net device has a single RX/TX pair, then the former has index 0 and the latter 1. When
// the device has multiqueue support, then RX queues have indices 2k, and TX queues 2k+1.
const RXQ_INDEX: u16 = 0;
const TXQ_INDEX: u16 = 1;

#[derive(Debug)]
pub enum Error {
    Virtio(crate::virtio::Error),
    Tap(tap::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct NetArgs {
    pub tap_name: String,
}
