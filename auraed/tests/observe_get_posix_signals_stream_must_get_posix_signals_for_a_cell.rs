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
async fn observe_get_posix_signal_stream_must_get_posix_signals_for_a_cell() {
    skip_if_not_root!("must_get_posix_signals_for_a_cell");
    skip_if_seccomp!("must_get_posix_signals_for_a_cell");

    let client = common::auraed_client().await;

    // Allocate a cell
    let cell1_name = retry!(
        client.allocate(CellServiceAllocateRequestBuilder::new().build()).await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Start an executable
    let exe1_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
    let pid1 = retry!(
        client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell1_name.clone())
                    .executable_name(exe1_name.clone())
                    .build(),
            )
            .await
    )
    .unwrap()
    .into_inner()
    .pid;

    // Allocate a second cell
    let cell2_name = retry!(
        client.allocate(CellServiceAllocateRequestBuilder::new().build()).await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Start a second executable
    let exe2_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
    let pid2 = retry!(
        client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell2_name.clone())
                    .executable_name(exe2_name.clone())
                    .build(),
            )
            .await
    )
    .unwrap()
    .into_inner()
    .pid;

    // Start intercepting signals for the first cell
    let intercepted_signals = intercept_posix_signals_stream(
        &client,
        GetPosixSignalsStreamRequestBuilder::new()
            .cell_workload(cell1_name.clone())
            .build(),
    )
    .await;

    // Stop executable in the first cell
    let _ = retry!(
        client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell1_name.clone()),
                executable_name: exe1_name.clone(),
            })
            .await
    );

    // Stop executable in the second cell
    let _ = retry!(
        client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell2_name.clone()),
                executable_name: exe2_name.clone(),
            })
            .await
    );

    // Wait for a little for the signals to arrive
    tokio::time::sleep(Duration::from_millis(500)).await;

    let guard = intercepted_signals.lock().await;

    // Assert we intercepted the signal for the executable in the first cell
    let expected = Signal { process_id: pid1, signal: 9 };
    assert!(
        guard.contains(&expected),
        "signal not found\nexpected: {expected:#?}\nintercepted: {guard:#?}",
    );
    // Assert we did NOT intercept the signal for the executable in the second cell
    assert!(
        !guard.contains(&Signal { process_id: pid2, signal: 9 }),
        "unexpected signal intercepted"
    );
}