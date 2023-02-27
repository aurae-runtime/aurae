// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::borrow::{Borrow, BorrowMut};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use virtio_device::{VirtioConfig, VirtioDeviceActions, VirtioDeviceType, VirtioMmioDevice};
use virtio_queue::Queue;
use vm_device::bus::MmioAddress;
use vm_device::device_manager::MmioManager;
use vm_device::{DeviceMmio, MutDeviceMmio};
use vm_memory::GuestAddressSpace;

use crate::virtio::features::{VIRTIO_F_IN_ORDER, VIRTIO_F_RING_EVENT_IDX, VIRTIO_F_VERSION_1};
use crate::virtio::net::features::*;
use crate::virtio::net::{Error, NetArgs, Result, NET_DEVICE_ID, VIRTIO_NET_HDR_SIZE};
use crate::virtio::{CommonConfig, Env, SingleFdSignalQueue, QUEUE_MAX_SIZE};

use super::bindings;
use super::queue_handler::QueueHandler;
use super::simple_handler::SimpleHandler;
use super::tap::Tap;

pub struct Net<M: GuestAddressSpace> {
    cfg: CommonConfig<M>,
    tap_name: String,
}

impl<M> Net<M>
where
    M: GuestAddressSpace + Clone + Send + 'static,
{
    pub fn new<B>(env: &mut Env<M, B>, args: &NetArgs) -> Result<Arc<Mutex<Self>>>
    where
        // We're using this (more convoluted) bound so we can pass both references and smart
        // pointers such as mutex guards here.
        B: DerefMut,
        B::Target: MmioManager<D = Arc<dyn DeviceMmio + Send + Sync>>,
    {
        let device_features = (1 << VIRTIO_F_VERSION_1)
            | (1 << VIRTIO_F_RING_EVENT_IDX)
            | (1 << VIRTIO_F_IN_ORDER)
            | (1 << VIRTIO_NET_F_CSUM)
            | (1 << VIRTIO_NET_F_GUEST_CSUM)
            | (1 << VIRTIO_NET_F_GUEST_TSO4)
            | (1 << VIRTIO_NET_F_GUEST_TSO6)
            | (1 << VIRTIO_NET_F_GUEST_UFO)
            | (1 << VIRTIO_NET_F_HOST_TSO4)
            | (1 << VIRTIO_NET_F_HOST_TSO6)
            | (1 << VIRTIO_NET_F_HOST_UFO);

        // An rx/tx queue pair.
        let queues = vec![
            Queue::new(env.mem.clone(), QUEUE_MAX_SIZE),
            Queue::new(env.mem.clone(), QUEUE_MAX_SIZE),
        ];

        // TODO: We'll need a minimal config space to support setting an explicit MAC addr
        // on the guest interface at least. We use an empty one for now.
        let config_space = Vec::new();
        let virtio_cfg = VirtioConfig::new(device_features, queues, config_space);

        let common_cfg = CommonConfig::new(virtio_cfg, env).map_err(Error::Virtio)?;

        let net = Arc::new(Mutex::new(Net {
            cfg: common_cfg,
            tap_name: args.tap_name.clone(),
        }));

        env.register_mmio_device(net.clone())
            .map_err(Error::Virtio)?;

        Ok(net)
    }
}

impl<M: GuestAddressSpace + Clone + Send + 'static> VirtioDeviceType for Net<M> {
    fn device_type(&self) -> u32 {
        NET_DEVICE_ID
    }
}

impl<M: GuestAddressSpace + Clone + Send + 'static> Borrow<VirtioConfig<M>> for Net<M> {
    fn borrow(&self) -> &VirtioConfig<M> {
        &self.cfg.virtio
    }
}

impl<M: GuestAddressSpace + Clone + Send + 'static> BorrowMut<VirtioConfig<M>> for Net<M> {
    fn borrow_mut(&mut self) -> &mut VirtioConfig<M> {
        &mut self.cfg.virtio
    }
}

impl<M: GuestAddressSpace + Clone + Send + 'static> VirtioDeviceActions for Net<M> {
    type E = Error;

    fn activate(&mut self) -> Result<()> {
        let tap = Tap::open_named(self.tap_name.as_str()).map_err(Error::Tap)?;

        // Set offload flags to match the relevant virtio features of the device (for now,
        // statically set in the constructor.
        tap.set_offload(
            bindings::TUN_F_CSUM
                | bindings::TUN_F_UFO
                | bindings::TUN_F_TSO4
                | bindings::TUN_F_TSO6,
        )
        .map_err(Error::Tap)?;

        // The layout of the header is specified in the standard and is 12 bytes in size. We
        // should define this somewhere.
        tap.set_vnet_hdr_size(VIRTIO_NET_HDR_SIZE as i32)
            .map_err(Error::Tap)?;

        let driver_notify = SingleFdSignalQueue {
            irqfd: self.cfg.irqfd.clone(),
            interrupt_status: self.cfg.virtio.interrupt_status.clone(),
        };

        let mut ioevents = self.cfg.prepare_activate().map_err(Error::Virtio)?;

        let rxq = self.cfg.virtio.queues.remove(0);
        let txq = self.cfg.virtio.queues.remove(0);
        let inner = SimpleHandler::new(driver_notify, rxq, txq, tap);

        let handler = Arc::new(Mutex::new(QueueHandler {
            inner,
            rx_ioevent: ioevents.remove(0),
            tx_ioevent: ioevents.remove(0),
        }));

        self.cfg.finalize_activate(handler).map_err(Error::Virtio)
    }

    fn reset(&mut self) -> std::result::Result<(), Error> {
        // Not implemented for now.
        Ok(())
    }
}

impl<M: GuestAddressSpace + Clone + Send + 'static> VirtioMmioDevice<M> for Net<M> {}

impl<M: GuestAddressSpace + Clone + Send + 'static> MutDeviceMmio for Net<M> {
    fn mmio_read(&mut self, _base: MmioAddress, offset: u64, data: &mut [u8]) {
        self.read(offset, data);
    }

    fn mmio_write(&mut self, _base: MmioAddress, offset: u64, data: &[u8]) {
        self.write(offset, data);
    }
}
