// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::fs::File;
use std::result;

use log::warn;
use virtio_blk::request::Request;
use virtio_blk::stdio_executor::{self, StdIoBackend};
use virtio_queue::{DescriptorChain, Queue};
use vm_memory::{self, GuestAddressSpace};

use crate::virtio::SignalUsedQueue;

#[derive(Debug)]
pub enum Error {
    GuestMemory(vm_memory::GuestMemoryError),
    Queue(virtio_queue::Error),
    ProcessRequest(stdio_executor::ProcessReqError),
}

impl From<vm_memory::GuestMemoryError> for Error {
    fn from(e: vm_memory::GuestMemoryError) -> Self {
        Error::GuestMemory(e)
    }
}

impl From<virtio_queue::Error> for Error {
    fn from(e: virtio_queue::Error) -> Self {
        Error::Queue(e)
    }
}

impl From<stdio_executor::ProcessReqError> for Error {
    fn from(e: stdio_executor::ProcessReqError) -> Self {
        Error::ProcessRequest(e)
    }
}

// This object is used to process the queue of a block device without making any assumptions
// about the notification mechanism. We're using a specific backend for now (the `StdIoBackend`
// object), but the aim is to have a way of working with generic backends and turn this into
// a more flexible building block. The name comes from processing and returning descriptor
// chains back to the device in the same order they are received.
pub struct InOrderQueueHandler<M: GuestAddressSpace, S: SignalUsedQueue> {
    pub driver_notify: S,
    pub queue: Queue<M>,
    pub disk: StdIoBackend<File>,
}

impl<M, S> InOrderQueueHandler<M, S>
where
    M: GuestAddressSpace,
    S: SignalUsedQueue,
{
    fn process_chain(&mut self, mut chain: DescriptorChain<M::T>) -> result::Result<(), Error> {
        let used_len = match Request::parse(&mut chain) {
            Ok(request) => self.disk.process_request(chain.memory(), &request)?,
            Err(e) => {
                warn!("block request parse error: {:?}", e);
                0
            }
        };

        self.queue.add_used(chain.head_index(), used_len)?;

        if self.queue.needs_notification()? {
            self.driver_notify.signal_used_queue(0);
        }

        Ok(())
    }

    pub fn process_queue(&mut self) -> result::Result<(), Error> {
        // To see why this is done in a loop, please look at the `Queue::enable_notification`
        // comments in `virtio_queue`.
        loop {
            self.queue.disable_notification()?;

            while let Some(chain) = self.queue.iter()?.next() {
                self.process_chain(chain)?;
            }

            if !self.queue.enable_notification()? {
                break;
            }
        }

        Ok(())
    }
}

// TODO: Figure out which unit tests make sense to add after implementing a generic backend
// abstraction for `InOrderHandler`.
