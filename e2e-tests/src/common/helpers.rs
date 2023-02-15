use std::sync::Arc;

use client::observe::observe_service::ObserveServiceClient;
use client::Client;

use proto::observe::{GetPosixSignalsStreamRequest, Signal};
use tokio::sync::Mutex;

pub async fn intercept_posix_signals_stream(
    client: &Client,
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
