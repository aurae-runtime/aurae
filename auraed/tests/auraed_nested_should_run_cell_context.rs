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

mod common;

use std::{
    fs::OpenOptions,
    os::unix::fs::FileTypeExt,
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};

use hyper_util::rt::TokioIo;
use proto::discovery::{
    DiscoverRequest, discovery_service_client::DiscoveryServiceClient,
};
use test_helpers::*;
use test_helpers_macros::shared_runtime_test;
use tokio::net::UnixStream;
use tokio::time::sleep;
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

#[shared_runtime_test]
async fn auraed_nested_flag_should_run_cell_context() {
    skip_if_not_root!("auraed_nested_flag_should_run_cell_context");
    skip_if_seccomp!("auraed_nested_flag_should_run_cell_context");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = tempdir.path().join("runtime");
    let library_dir = tempdir.path().join("library");
    let log_path = tempdir.path().join("auraed.log");
    std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
    std::fs::create_dir_all(&library_dir).expect("library dir");

    let child = spawn_auraed_nested(&runtime_dir, &library_dir, &log_path);
    let _guard = common::ChildGuard::new(child);

    // In nested (cell) context TLS is disabled, so a plaintext Unix client should work.
    let channel = wait_for_socket_and_connect(
        &runtime_dir.join("aurae.sock"),
        &log_path,
        Duration::from_secs(20),
    )
    .await;
    let mut discovery = DiscoveryServiceClient::new(channel);
    discovery
        .discover(DiscoverRequest {})
        .await
        .expect("discover over Unix socket without TLS");
}

fn spawn_auraed_nested(
    runtime_dir: &Path,
    library_dir: &Path,
    log_path: &Path,
) -> Child {
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .expect("open log file");

    Command::new(env!("CARGO_BIN_EXE_auraed"))
        .arg("--runtime-dir")
        .arg(runtime_dir.to_str().expect("runtime dir"))
        .arg("--library-dir")
        .arg(library_dir.to_str().expect("library dir"))
        .arg("--nested")
        .stdout(Stdio::from(log.try_clone().expect("clone log file")))
        .stderr(Stdio::from(log))
        .spawn()
        .expect("spawn auraed")
}

async fn wait_for_socket_and_connect(
    path: &Path,
    log_path: &Path,
    timeout: Duration,
) -> Channel {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if path.exists() {
            if let Ok(channel) = connect_unix(path).await {
                let meta = std::fs::symlink_metadata(path)
                    .expect("metadata for socket");
                assert!(
                    meta.file_type().is_socket(),
                    "expected {:?} to be a Unix socket",
                    path
                );
                return channel;
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    let logs = std::fs::read_to_string(log_path).unwrap_or_default();
    panic!(
        "socket {path:?} not ready within {:?}. auraed logs:\n{}",
        timeout, logs
    );
}

async fn connect_unix(path: &Path) -> Result<Channel, tonic::transport::Error> {
    let path = path.to_owned();
    Endpoint::try_from("http://[::]:50051")
        .expect("endpoint")
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = UnixStream::connect(path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
}
