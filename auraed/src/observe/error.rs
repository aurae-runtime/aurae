/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

use proto::observe::LogChannelType;
use thiserror::Error;
use tonic::Status;
use tracing::error;

#[derive(Debug, Error)]
pub enum ObserveServiceError {
    #[error("Channel already registered with type {channel_type:?} for {pid}")]
    ChannelAlreadyRegistered { pid: i32, channel_type: LogChannelType },
    #[error("Failed to find any registered channels for {pid}")]
    NoChannelsForPid { pid: i32 },
    #[error("Failed to find channel type {channel_type:?} for {pid}")]
    ChannelNotRegistered { pid: i32, channel_type: LogChannelType },
    #[error("{channel_type} is not a valid LogChannelType")]
    InvalidLogChannelType { channel_type: i32 },
}

impl From<ObserveServiceError> for Status {
    fn from(err: ObserveServiceError) -> Self {
        let msg = err.to_string();
        error!("{msg}");
        match err {
            ObserveServiceError::ChannelAlreadyRegistered { .. } => {
                Status::internal(msg)
            }
            ObserveServiceError::NoChannelsForPid { .. }
            | ObserveServiceError::ChannelNotRegistered { .. } => {
                Status::not_found(msg)
            }
            ObserveServiceError::InvalidLogChannelType { .. } => {
                Status::invalid_argument(msg)
            }
        }
    }
}