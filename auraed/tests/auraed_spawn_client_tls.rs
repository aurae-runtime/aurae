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
use client::{
    AuraeConfig, AuraeSocket, AuthConfig, Client, ClientError, SystemConfig,
};
use proto::discovery::DiscoverRequest;
use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use test_helpers::*;
use tokio::time::{Duration, sleep};

mod common;

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
            client_crt: material.client_crt.to_string_lossy().into_owned(),
            client_key: material.client_key.to_string_lossy().into_owned(),
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

struct TlsMaterial {
    ca_crt: PathBuf,
    server_crt: PathBuf,
    server_key: PathBuf,
    client_crt: PathBuf,
    client_key: PathBuf,
}

fn generate_tls_material(dir: &Path) -> TlsMaterial {
    let ca_crt = dir.join("ca.crt");
    let ca_key = dir.join("ca.key");
    let mut ca_cmd = Command::new("openssl");
    ca_cmd
        .arg("req")
        .arg("-x509")
        .arg("-nodes")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-sha256")
        .arg("-days")
        .arg("365")
        .arg("-keyout")
        .arg(&ca_key)
        .arg("-out")
        .arg(&ca_crt)
        .arg("-subj")
        .arg("/CN=AuraeTestCA");
    run_openssl(ca_cmd);

    let server_key = dir.join("server.key");
    let server_csr = dir.join("server.csr");
    let mut server_req = Command::new("openssl");
    server_req
        .arg("req")
        .arg("-new")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-nodes")
        .arg("-keyout")
        .arg(&server_key)
        .arg("-out")
        .arg(&server_csr)
        .arg("-subj")
        .arg("/CN=server.unsafe.aurae.io")
        .arg("-addext")
        .arg("subjectAltName = DNS:server.unsafe.aurae.io");
    run_openssl(server_req);

    let server_crt = dir.join("_signed.server.crt");
    let server_ext = dir.join("server.ext");
    std::fs::write(
        &server_ext,
        "subjectAltName = DNS:server.unsafe.aurae.io\nextendedKeyUsage = serverAuth\n",
    )
    .expect("write server ext");
    let mut server_sign = Command::new("openssl");
    server_sign
        .arg("x509")
        .arg("-req")
        .arg("-days")
        .arg("365")
        .arg("-in")
        .arg(&server_csr)
        .arg("-CA")
        .arg(&ca_crt)
        .arg("-CAkey")
        .arg(&ca_key)
        .arg("-CAcreateserial")
        .arg("-out")
        .arg(&server_crt)
        .arg("-extfile")
        .arg(&server_ext);
    run_openssl(server_sign);

    let client_key = dir.join("client.key");
    let client_csr = dir.join("client.csr");
    let mut client_req = Command::new("openssl");
    client_req
        .arg("req")
        .arg("-new")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-nodes")
        .arg("-keyout")
        .arg(&client_key)
        .arg("-out")
        .arg(&client_csr)
        .arg("-subj")
        .arg("/CN=client.unsafe.aurae.io")
        .arg("-addext")
        .arg("subjectAltName = DNS:client.unsafe.aurae.io");
    run_openssl(client_req);

    let client_crt = dir.join("_signed.client.crt");
    let client_ext = dir.join("client.ext");
    std::fs::write(
        &client_ext,
        "subjectAltName = DNS:client.unsafe.aurae.io\nextendedKeyUsage = clientAuth\n",
    )
    .expect("write client ext");
    let mut client_sign = Command::new("openssl");
    client_sign
        .arg("x509")
        .arg("-req")
        .arg("-days")
        .arg("365")
        .arg("-in")
        .arg(&client_csr)
        .arg("-CA")
        .arg(&ca_crt)
        .arg("-CAkey")
        .arg(&ca_key)
        .arg("-CAcreateserial")
        .arg("-out")
        .arg(&client_crt)
        .arg("-extfile")
        .arg(&client_ext);
    run_openssl(client_sign);

    TlsMaterial { ca_crt, server_crt, server_key, client_crt, client_key }
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

fn run_openssl(mut cmd: Command) {
    let status = cmd.status().expect("failed to run openssl");
    assert!(
        status.success(),
        "openssl command {:?} failed with status {status:?}",
        cmd
    );
}
