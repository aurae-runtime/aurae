// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use event_manager::{EventOps, Events, MutEventSubscriber};
use log::error;
use vm_memory::GuestAddressSpace;
use vmm_sys_util::epoll::EventSet;
use vmm_sys_util::eventfd::EventFd;

use crate::virtio::SingleFdSignalQueue;

use super::simple_handler::SimpleHandler;

const TAPFD_DATA: u32 = 0;
const RX_IOEVENT_DATA: u32 = 1;
const TX_IOEVENT_DATA: u32 = 2;

pub struct QueueHandler<M: GuestAddressSpace> {
    pub inner: SimpleHandler<M, SingleFdSignalQueue>,
    pub rx_ioevent: EventFd,
    pub tx_ioevent: EventFd,
}

impl<M: GuestAddressSpace> QueueHandler<M> {
    // Helper method that receives an error message to be logged and the `ops` handle
    // which is used to unregister all events.
    fn handle_error<S: AsRef<str>>(&self, s: S, ops: &mut EventOps) {
        error!("{}", s.as_ref());
        ops.remove(Events::empty(&self.rx_ioevent))
            .expect("Failed to remove rx ioevent");
        ops.remove(Events::empty(&self.tx_ioevent))
            .expect("Failed to remove tx ioevent");
        ops.remove(Events::empty(&self.inner.tap))
            .expect("Failed to remove tap event");
    }
}

impl<M: GuestAddressSpace> MutEventSubscriber for QueueHandler<M> {
    fn process(&mut self, events: Events, ops: &mut EventOps) {
        // TODO: We can also consider panicking on the errors that cannot be generated
        // or influenced.

        if events.event_set() != EventSet::IN {
            self.handle_error("Unexpected event_set", ops);
            return;
        }

        match events.data() {
            TAPFD_DATA => {
                if let Err(e) = self.inner.process_tap() {
                    self.handle_error(format!("Process tap error {:?}", e), ops);
                }
            }
            RX_IOEVENT_DATA => {
                if self.rx_ioevent.read().is_err() {
                    self.handle_error("Rx ioevent read", ops);
                } else if let Err(e) = self.inner.process_rxq() {
                    self.handle_error(format!("Process rx error {:?}", e), ops);
                }
            }
            TX_IOEVENT_DATA => {
                if self.tx_ioevent.read().is_err() {
                    self.handle_error("Tx ioevent read", ops);
                }
                if let Err(e) = self.inner.process_txq() {
                    self.handle_error(format!("Process tx error {:?}", e), ops);
                }
            }
            _ => self.handle_error("Unexpected data", ops),
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        ops.add(Events::with_data(
            &self.inner.tap,
            TAPFD_DATA,
            EventSet::IN | EventSet::EDGE_TRIGGERED,
        ))
        .expect("Unable to add tapfd");

        ops.add(Events::with_data(
            &self.rx_ioevent,
            RX_IOEVENT_DATA,
            EventSet::IN,
        ))
        .expect("Unable to add rxfd");

        ops.add(Events::with_data(
            &self.tx_ioevent,
            TX_IOEVENT_DATA,
            EventSet::IN,
        ))
        .expect("Unable to add txfd");
    }
}
