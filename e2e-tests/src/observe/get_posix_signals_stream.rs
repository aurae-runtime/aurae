#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use aurae_client::{
        cells::cell_service::CellServiceClient,
        observe::observe_service::ObserveServiceClient, AuraeClient,
    };
    use aurae_proto::{
        cells::{
            Cell, CellServiceAllocateRequest, CellServiceStartRequest,
            CellServiceStartResponse, CellServiceStopRequest, Executable,
        },
        observe::{GetPosixSignalsStreamRequest, Signal},
    };
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn get_posix_signals_for_daemon() {
        let client = AuraeClient::default().await;
        let client = client.expect("failed to initialize aurae-client");

        let cell_name = format!("ae-e2e-{}", uuid::Uuid::new_v4());
        let res = client
            .allocate(CellServiceAllocateRequest {
                cell: Some(Cell {
                    name: cell_name.clone(),
                    cpu: None,
                    cpuset: None,
                    memory: None,
                    isolate_network: false,
                    isolate_process: false,
                }),
            })
            .await;
        assert!(res.is_ok());

        let exe_name = String::from("ae-e2e-sleeper");
        let res = client
            .start(CellServiceStartRequest {
                cell_name: Some(cell_name.clone()),
                executable: Some(Executable {
                    name: exe_name.clone(),
                    command: String::from("sleep 400"),
                    description: String::from(
                        "get_posix_signals_for_daemon sleeper",
                    ),
                }),
            })
            .await;
        assert!(res.is_ok());

        let pid = res.expect("CellServiceStartResponse").into_inner().pid;

        let res = client
            .get_posix_signals_stream(GetPosixSignalsStreamRequest {})
            .await;
        assert!(res.is_ok());

        let mut signals =
            res.expect("GetPosixSignalsStreamResponse").into_inner();

        let intercepted = Arc::new(Mutex::new(Vec::new()));
        let intercepted_in_thread = intercepted.clone();
        let _ = tokio::spawn(async move {
            while let Some(res) =
                futures_util::StreamExt::next(&mut signals).await
            {
                let res = res.expect("signal");
                let mut guard = intercepted_in_thread.lock().await;
                guard.push(res.signal.expect("signal"));
            }
        });

        let res = client
            .stop(CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: exe_name.clone(),
            })
            .await;
        assert!(res.is_ok());

        let expected_signal = Signal { process_id: pid as i64, signal: 9 };

        tokio::time::sleep(Duration::from_millis(500)).await;

        let intercepted_local = intercepted.clone();
        let guard = intercepted_local.lock().await;

        assert!(guard.contains(&expected_signal), "signal not found");
    }

    #[test]
    fn get_posix_signals_for_contains() {
        assert!(true);
    }
}
