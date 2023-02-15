#[cfg(test)]
mod tests {
    use crate::common::{
        helpers::intercept_posix_signals_stream,
        request_builders::{
            CellServiceAllocateRequestBuilder, CellServiceStartRequestBuilder,
            GetPosixSignalsStreamRequestBuilder,
        },
    };
    use client::{
        cells::cell_service::CellServiceClient, Client as AuraeClient,
    };
    use proto::{cells::CellServiceStopRequest, observe::Signal};
    use std::time::Duration;

    #[tokio::test]
    #[ignore = "we can not run eBPF tests in Github actions"]
    async fn must_get_posix_signals_for_the_host() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start an executable
        let exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell_name.clone())
                    .executable_name(exe_name.clone())
                    .build(),
            )
            .await
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
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            })
            .await;

        // Wait for a little for the signal to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert we intercepted the signal
        let guard = intercepted_signals.lock().await;
        let expected = Signal { process_id: pid, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
    }

    #[tokio::test]
    #[ignore = "we can not run eBPF tests in Github actions"]
    async fn must_get_posix_signals_for_a_cell() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start an executable
        let exe1_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid1 = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell1_name.clone())
                    .executable_name(exe1_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .pid;

        // Allocate a second cell
        let cell2_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start a second executable
        let exe2_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid2 = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell2_name.clone())
                    .executable_name(exe2_name.clone())
                    .build(),
            )
            .await
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
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell1_name.clone()),
                executable_name: exe1_name.clone(),
            })
            .await;

        // Stop executable in the second cell
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell2_name.clone()),
                executable_name: exe2_name.clone(),
            })
            .await;

        // Wait for a little for the signals to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        let guard = intercepted_signals.lock().await;

        // Assert we intercepted the signal for the executable in the first cell
        let expected = Signal { process_id: pid1, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
        // Assert we did NOT intercept the signal for the executable in the second cell
        assert!(
            !guard.contains(&Signal { process_id: pid2, signal: 9 }),
            "unexpected signal intercepted"
        );
    }

    #[tokio::test]
    #[ignore = "we can not run eBPF tests in Github actions"]
    async fn must_get_posix_signals_for_a_nested_cell() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell
        let cell1_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start an executable
        let exe1_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid1 = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell1_name.clone())
                    .executable_name(exe1_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .pid;

        // Allocate a nested cell
        let nested_cell_name = client
            .allocate(
                CellServiceAllocateRequestBuilder::new()
                    .parent_cell_name(cell1_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start an executable in the nested cell
        let nested_exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let nested_pid = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(nested_cell_name.clone())
                    .executable_name(nested_exe_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .pid;

        // Allocate a second cell
        let cell2_name = client
            .allocate(CellServiceAllocateRequestBuilder::new().build())
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start a second executable
        let exe2_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let pid2 = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell2_name.clone())
                    .executable_name(exe2_name.clone())
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .pid;

        // Start intercepting signals for the nested cell
        let intercepted_signals = intercept_posix_signals_stream(
            &client,
            GetPosixSignalsStreamRequestBuilder::new()
                .cell_workload(nested_cell_name.clone())
                .build(),
        )
        .await;

        // Stop executable in the first cell
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell1_name.clone()),
                executable_name: exe1_name.clone(),
            })
            .await;

        // Stop executable in the nested cell
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(nested_cell_name.clone()),
                executable_name: nested_exe_name.clone(),
            })
            .await;

        // Stop executable in the second cell
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell2_name.clone()),
                executable_name: exe2_name.clone(),
            })
            .await;

        // Wait for a little for the signals to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        let guard = intercepted_signals.lock().await;

        // Assert we intercepted the signal for the executable in the nested cell
        let expected = Signal { process_id: nested_pid, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
        // Assert we did NOT intercept the signal for the executable in the first (parent) cell
        assert!(
            !guard.contains(&Signal { process_id: pid1, signal: 9 }),
            "unexpected signal intercepted"
        );
        // Assert we did NOT intercept the signal for the executable in the second cell
        assert!(
            !guard.contains(&Signal { process_id: pid2, signal: 9 }),
            "unexpected signal intercepted"
        );
    }

    #[tokio::test]
    #[ignore = "placeholder test for a follow-up pid mapping issue"]
    async fn must_map_host_pids_to_namespace_pids() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        // Allocate a cell and unshare the PID namespace
        let cell_name = client
            .allocate(
                CellServiceAllocateRequestBuilder::new()
                    .isolate_process()
                    .build(),
            )
            .await
            .unwrap()
            .into_inner()
            .cell_name;

        // Start an executable
        let exe_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let nspid = client
            .start(
                CellServiceStartRequestBuilder::new()
                    .cell_name(cell_name.clone())
                    .executable_name(exe_name.clone())
                    .build(),
            )
            .await
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
        _ = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            })
            .await;

        // Wait for a little for the signal to arrive
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert we intercepted the signal
        let guard = intercepted_signals.lock().await;
        let expected = Signal { process_id: nspid, signal: 9 };
        assert!(
            guard.contains(&expected),
            "signal not found\nexpected: {:#?}\nintercepted: {:#?}",
            expected,
            guard
        );
    }
}
