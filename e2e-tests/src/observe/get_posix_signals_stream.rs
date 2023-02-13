#[cfg(test)]
mod tests {
    use std::time::Duration;

    use client::Client as AuraeClient;
    use proto::{cells::CellServiceStopRequest, observe::Signal};

    use crate::observe::{
        helpers::{
            allocate_cell, intercept_posix_signals_stream, start_in_cell,
            stop_in_cell,
        },
        request_builders::{
            CellServiceAllocateRequestBuilder, CellServiceStartRequestBuilder,
            GetPosixSignalsStreamRequestBuilder,
        },
    };

    #[tokio::test]
    async fn must_get_posix_signals_for_the_host() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Start an executable
        let exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell_name.clone())
                .executable_name(exe_name.clone())
                .build(),
        )
        .await;

        // Start intercepting POSIX signals for the host
        let intercepted_signals = intercept_posix_signals_stream(
            &client,
            GetPosixSignalsStreamRequestBuilder::new().build(),
        )
        .await;

        // Stop the executable (should trigger SIGKILL)
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            },
        )
        .await;

        // Wait for a little for the signal to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert we intercepted the signal
        let guard = intercepted_signals.lock().await;
        let expected = Signal { process_id: pid as i64, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
    }

    #[tokio::test]
    async fn must_get_posix_signals_for_a_cell() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Start an executable
        let exe1_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid1 = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell1_name.clone())
                .executable_name(exe1_name.clone())
                .build(),
        )
        .await;

        // Allocate a second cell
        let cell2_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Start a second executable
        let exe2_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid2 = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell2_name.clone())
                .executable_name(exe2_name.clone())
                .build(),
        )
        .await;

        // Start intercepting signals for the first cell
        let intercepted_signals = intercept_posix_signals_stream(
            &client,
            GetPosixSignalsStreamRequestBuilder::new()
                .cell_workload(cell1_name.clone())
                .build(),
        )
        .await;

        // Stop executable in the first cell
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell1_name.clone()),
                executable_name: exe1_name.clone(),
            },
        )
        .await;

        // Stop executable in the second cell
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell2_name.clone()),
                executable_name: exe2_name.clone(),
            },
        )
        .await;

        // Wait for a little for the signals to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        let guard = intercepted_signals.lock().await;

        // Assert we intercepted the signal for the executable in the first cell
        let expected = Signal { process_id: pid1 as i64, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
        // Assert we did NOT intercept the signal for the executable in the second cell
        assert!(
            !guard.contains(&Signal { process_id: pid2 as i64, signal: 9 }),
            "unexpected signal intercepted"
        );
    }

    #[tokio::test]
    async fn must_get_posix_signals_for_a_nested_cell() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Start an executable
        let exe1_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid1 = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell1_name.clone())
                .executable_name(exe1_name.clone())
                .build(),
        )
        .await;

        // Allocate a nested cell
        let nested_cell_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new()
                .parent_cell_name(cell1_name.clone())
                .build(),
        )
        .await;

        // Start an executable in the nested cell
        let nested_exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let nested_pid = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(nested_cell_name.clone())
                .executable_name(nested_exe_name.clone())
                .build(),
        )
        .await;

        // Allocate a second cell
        let cell2_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().build(),
        )
        .await;

        // Start a second executable
        let exe2_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid2 = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell2_name.clone())
                .executable_name(exe2_name.clone())
                .build(),
        )
        .await;

        // Start intercepting signals for the nested cell
        let intercepted_signals = intercept_posix_signals_stream(
            &client,
            GetPosixSignalsStreamRequestBuilder::new()
                .cell_workload(nested_cell_name.clone())
                .build(),
        )
        .await;

        // Stop executable in the first cell
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell1_name.clone()),
                executable_name: exe1_name.clone(),
            },
        )
        .await;

        // Stop executable in the nested cell
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(nested_cell_name.clone()),
                executable_name: nested_exe_name.clone(),
            },
        )
        .await;

        // Stop executable in the second cell
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell2_name.clone()),
                executable_name: exe2_name.clone(),
            },
        )
        .await;

        // Wait for a little for the signals to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        let guard = intercepted_signals.lock().await;

        // Assert we intercepted the signal for the executable in the nested cell
        let expected = Signal { process_id: nested_pid as i64, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
        // Assert we did NOT intercept the signal for the executable in the first (parent) cell
        assert!(
            !guard.contains(&Signal { process_id: pid1 as i64, signal: 9 }),
            "unexpected signal intercepted"
        );
        // Assert we did NOT intercept the signal for the executable in the second cell
        assert!(
            !guard.contains(&Signal { process_id: pid2 as i64, signal: 9 }),
            "unexpected signal intercepted"
        );
    }

    #[tokio::test]
    #[ignore] // This is for a different PR :)
    async fn must_map_host_pids_to_namespace_pids() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell and unshare the PID namespace
        let cell_name = allocate_cell(
            &client,
            CellServiceAllocateRequestBuilder::new().isolate_process().build(),
        )
        .await;

        // Start an executable
        let exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let nspid = start_in_cell(
            &client,
            CellServiceStartRequestBuilder::new()
                .cell_name(cell_name.clone())
                .executable_name(exe_name.clone())
                .build(),
        )
        .await;

        // Start intercepting POSIX signals for the host
        let intercepted_signals = intercept_posix_signals_stream(
            &client,
            GetPosixSignalsStreamRequestBuilder::new().build(),
        )
        .await;

        // Stop the executable (should trigger SIGKILL)
        stop_in_cell(
            &client,
            CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            },
        )
        .await;

        // Wait for a little for the signal to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert we intercepted the signal
        let guard = intercepted_signals.lock().await;
        let expected = Signal { process_id: nspid as i64, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
    }
}
