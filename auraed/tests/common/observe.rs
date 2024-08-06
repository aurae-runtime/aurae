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
#![allow(unused)]

use crate::retry;
use client::{observe::observe_service::ObserveServiceClient, Client};
use proto::observe::{
    GetPosixSignalsStreamRequest, Signal, Workload, WorkloadType,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn intercept_posix_signals_stream(
    client: &Client,
    req: GetPosixSignalsStreamRequest,
) -> Arc<Mutex<Vec<Signal>>> {
    let res = retry!(client.get_posix_signals_stream(req.clone()).await);
    assert!(res.is_ok());

    let mut signals = res.expect("GetPosixSignalsStreamResponse").into_inner();

    let intercepted = Arc::new(Mutex::new(Vec::new()));
    let intercepted_in_thread = intercepted.clone();

    let _ignored = tokio::spawn(async move {
        while let Some(res) = futures_util::StreamExt::next(&mut signals).await
        {
            let res = res.expect("signal");
            let mut guard = intercepted_in_thread.lock().await;
            guard.push(res.signal.expect("signal"));
        }
    });

    intercepted
}

pub(crate) struct GetPosixSignalsStreamRequestBuilder {
    workload: Option<Workload>,
}

impl GetPosixSignalsStreamRequestBuilder {
    pub fn new() -> Self {
        Self { workload: None }
    }

    pub fn cell_workload(&mut self, name: String) -> &mut Self {
        self.workload = Some(Workload {
            workload_type: WorkloadType::Cell.into(),
            id: name,
        });
        self
    }

    pub fn build(&self) -> GetPosixSignalsStreamRequest {
        GetPosixSignalsStreamRequest { workload: self.workload.clone() }
    }
}