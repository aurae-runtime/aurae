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
use client::cells::cell_service::CellServiceClient;
use common::{
    cells::{
        CellServiceAllocateRequestBuilder, CellServiceStartRequestBuilder,
    },
    observe::{
        intercept_posix_signals_stream, GetPosixSignalsStreamRequestBuilder,
    },
};
use proto::{cells::CellServiceStopRequest, observe::Signal};
use std::time::Duration;
use test_helpers::*;

mod common;

#[test_helpers_macros::shared_runtime_test]
#[ignore = "we can not run eBPF tests in Github actions"]
async fn observe_get_posix_signal_stream_must_get_posix_signals_for_the_host() {
    skip_if_not_root!("must_get_posix_signals_for_the_host");
    skip_if_seccomp!("must_get_posix_signals_for_the_host");

    let client = common::auraed_client().await;

    // Allocate a cell
    let cell_name = retry!(
        client.allocate(CellServiceAllocateRequestBuilder::new().build()).await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Start an executable
    let exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
    let pid = retry!(
        client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell_name.clone())
                    .executable_name(exe_name.clone())
                    .build(),
            )
            .await
    )
    .unwrap()
    .into_inner()
    .pid;

    // Start intercepting POSIX signals for the host
    let intercepted_signals = intercept_posix_signals_stream(
        &client,
        GetPosixSignalsStreamRequestBuilder::new().build(),
    )
    .await;

    // Stop the executable (should trigger SIGKILL)
    let _ = retry!(
        client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            })
            .await
    );

    // Wait for a little for the signal to arrive
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Assert we intercepted the signal
    let guard = intercepted_signals.lock().await;
    let expected = Signal { process_id: pid, signal: 9 };
    assert!(
        guard.contains(&expected),
        "signal not found\nexpected: {expected:#?}\nintercepted: {guard:#?}",
    );
}