use std::sync::Arc;

use client::observe::observe_service::ObserveServiceClient;
use client::{cells::cell_service::CellServiceClient, Client as AuraeClient};
use proto::cells::{
    CellServiceAllocateRequest, CellServiceStartRequest, CellServiceStopRequest,
};
use proto::observe::{GetPosixSignalsStreamRequest, Signal};
use tokio::sync::Mutex;

pub async fn allocate_cell(
    client: &AuraeClient,
    req: CellServiceAllocateRequest,
) -> String {
    let res = client.allocate(req).await;
    assert!(res.is_ok());
    res.expect("CellServiceAllocateResponse").into_inner().cell_name
}

pub async fn start_in_cell(
    client: &AuraeClient,
    req: CellServiceStartRequest,
) -> i32 {
    let res = client.start(req).await;
    assert!(res.is_ok());
    res.expect("CellServiceStartResponse").into_inner().pid
}

pub async fn stop_in_cell(client: &AuraeClient, req: CellServiceStopRequest) {
    let res = client.stop(req).await;
    assert!(res.is_ok());
}

pub async fn intercept_posix_signals_stream(
    client: &AuraeClient,
    req: GetPosixSignalsStreamRequest,
) -> Arc<Mutex<Vec<Signal>>> {
    let res = client.get_posix_signals_stream(req).await;
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
