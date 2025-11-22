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

use crate::common::tls::{TlsMaterial, generate_server_and_client_tls};
use client::discovery::discovery_service::DiscoveryServiceClient;
use client::{AuraeConfig, AuraeSocket, AuthConfig, Client, SystemConfig};
use proto::discovery::DiscoverRequest;
use std::{
    fs::OpenOptions,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
use test_helpers::*;
use test_helpers_macros::shared_runtime_test;
use tokio::time::sleep;

#[shared_runtime_test]
async fn auraed_daemon_default_should_bind_runtime_unix_socket_and_accept_grpc()
{
    skip_if_not_root!(
        "auraed_daemon_default_should_bind_runtime_unix_socket_and_accept_grpc"
    );
    skip_if_seccomp!(
        "auraed_daemon_default_should_bind_runtime_unix_socket_and_accept_grpc"
    );

    let tempdir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = tempdir.path().join("runtime");
    let library_dir = tempdir.path().join("library");
    std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
    std::fs::create_dir_all(&library_dir).expect("library dir");

    let tls_dir = tempdir.path().join("pki");
    std::fs::create_dir_all(&tls_dir).expect("pki dir");
    let tls = generate_server_and_client_tls(&tls_dir);

    let log_path = tempdir.path().join("auraed.log");
    let auraed_child =
        spawn_auraed_daemon(&runtime_dir, &library_dir, &log_path, &tls);
    let mut guard = common::ChildGuard::new(auraed_child);

    wait_for_default_socket(
        &runtime_dir,
        &log_path,
        guard.child_mut().expect("auraed child"),
    )
    .await;

    let socket_path = runtime_dir.join("aurae.sock");
    let client_config = AuraeConfig {
        auth: AuthConfig {
            ca_crt: tls.ca_crt.to_string_lossy().into_owned(),
            client_crt: tls
                .client_crt
                .as_ref()
                .expect("client crt")
                .to_string_lossy()
                .into_owned(),
            client_key: tls
                .client_key
                .as_ref()
                .expect("client key")
                .to_string_lossy()
                .into_owned(),
        },
        system: SystemConfig { socket: AuraeSocket::Path(socket_path.into()) },
    };

    let client =
        Client::new(client_config).await.expect("TLS client over Unix socket");
    let _ = client
        .discover(DiscoverRequest {})
        .await
        .expect("discover should succeed on default Unix socket");
}

fn spawn_auraed_daemon(
    runtime_dir: &Path,
    library_dir: &Path,
    log_path: &Path,
    tls: &TlsMaterial,
) -> Child {
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .expect("open log file");

    Command::new(env!("CARGO_BIN_EXE_auraed"))
        .arg("--runtime-dir")
        .arg(runtime_dir)
        .arg("--library-dir")
        .arg(library_dir)
        .arg("--ca-crt")
        .arg(&tls.ca_crt)
        .arg("--server-crt")
        .arg(&tls.server_crt)
        .arg("--server-key")
        .arg(&tls.server_key)
        .stdout(Stdio::from(log.try_clone().expect("clone log file")))
        .stderr(Stdio::from(log))
        .spawn()
        .expect("spawn auraed")
}

async fn wait_for_default_socket(
    runtime_dir: &Path,
    log_path: &Path,
    child: &mut Child,
) {
    let socket_path = runtime_dir.join("aurae.sock");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);

    while tokio::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("check child status") {
            let logs = std::fs::read_to_string(log_path).unwrap_or_default();
            panic!("auraed exited early with status {status:?}. logs:\n{logs}");
        }

        if socket_path.exists() {
            let meta = std::fs::symlink_metadata(&socket_path)
                .expect("metadata for socket");
            assert!(
                meta.file_type().is_socket(),
                "expected {:?} to be a Unix socket",
                socket_path
            );
            let mode = meta.permissions().mode() & 0o777;
            assert_eq!(
                mode, 0o766,
                "expected socket mode 0o766, got {:o}",
                mode
            );
            return;
        }

        sleep(Duration::from_millis(100)).await;
    }

    let logs = std::fs::read_to_string(log_path).unwrap_or_default();
    panic!(
        "socket {:?} not ready within 20s. auraed logs:\n{}",
        socket_path, logs
    );
}
