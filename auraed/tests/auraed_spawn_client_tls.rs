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

use client::discovery::discovery_service::DiscoveryServiceClient;
use client::{AuraeConfig, AuraeSocket, AuthConfig, Client, SystemConfig};
use proto::discovery::DiscoverRequest;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use test_helpers::*;
use tokio::time::{Duration, sleep};

mod common;
use crate::common::tls::{TlsMaterial, generate_server_and_client_tls};

#[test_helpers_macros::shared_runtime_test]
async fn auraed_spawn_client_tls_enforces_mtls() {
    skip_if_not_root!("auraed_spawn_client_tls_enforces_mtls");
    skip_if_seccomp!("auraed_spawn_client_tls_enforces_mtls");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = tempdir.path().join("runtime");
    let library_dir = tempdir.path().join("library");
    std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
    std::fs::create_dir_all(&library_dir).expect("library dir");

    let tls_dir = tempdir.path().join("pki");
    std::fs::create_dir_all(&tls_dir).expect("pki dir");

    let material = generate_tls_material(&tls_dir);

    let listener =
        TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let socket_addr = listener.local_addr().expect("listener addr");
    drop(listener);

    let mut auraed_child = spawn_auraed(
        &material,
        socket_addr,
        runtime_dir.clone(),
        library_dir.clone(),
    );

    wait_for_listener(socket_addr, &mut auraed_child).await;

    let no_tls_client = Client::new_no_tls(AuraeSocket::Addr(socket_addr))
        .await
        .expect("plaintext client should connect");
    let err = no_tls_client
        .discover(DiscoverRequest {})
        .await
        .expect_err("discover should fail without TLS");
    match err.code() {
        tonic::Code::Unavailable | tonic::Code::Unknown => {}
        other => panic!(
            "expected TLS enforcement error, got status {other:?} ({err:?})"
        ),
    }

    let client_config = AuraeConfig {
        auth: AuthConfig {
            ca_crt: material.ca_crt.to_string_lossy().into_owned(),
            client_crt: material
                .client_crt
                .as_ref()
                .expect("client crt")
                .to_string_lossy()
                .into_owned(),
            client_key: material
                .client_key
                .as_ref()
                .expect("client key")
                .to_string_lossy()
                .into_owned(),
        },
        system: SystemConfig { socket: AuraeSocket::Addr(socket_addr) },
    };

    let client = Client::new(client_config)
        .await
        .expect("expected mTLS client to connect");
    let _ = client
        .discover(DiscoverRequest {})
        .await
        .expect("discover should succeed over mTLS");

    teardown_child(&mut auraed_child);
}

fn generate_tls_material(dir: &Path) -> TlsMaterial {
    generate_server_and_client_tls(dir)
}

fn spawn_auraed(
    material: &TlsMaterial,
    socket_addr: SocketAddr,
    runtime_dir: PathBuf,
    library_dir: PathBuf,
) -> Child {
    Command::new(env!("CARGO_BIN_EXE_auraed"))
        .arg("--server-crt")
        .arg(&material.server_crt)
        .arg("--server-key")
        .arg(&material.server_key)
        .arg("--ca-crt")
        .arg(&material.ca_crt)
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--library-dir")
        .arg(&library_dir)
        .arg("--socket")
        .arg(socket_addr.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn auraed")
}

async fn wait_for_listener(addr: SocketAddr, child: &mut Child) {
    for _ in 0..100 {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        if let Some(status) = child.try_wait().expect("check child status") {
            let mut stderr = String::new();
            if let Some(mut child_stderr) = child.stderr.take() {
                let _ = child_stderr.read_to_string(&mut stderr);
            }
            panic!(
                "auraed exited early with status {status:?}, stderr: {stderr}"
            );
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("server {addr} was not ready in time");
}

fn teardown_child(child: &mut Child) {
    if let Err(e) = child.kill() {
        if e.kind() != std::io::ErrorKind::InvalidInput {
            panic!("failed to kill auraed child: {e}");
        }
    }
    let _ = child.wait();
}
