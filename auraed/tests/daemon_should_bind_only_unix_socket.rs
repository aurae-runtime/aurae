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

//! Ensure auraed defaults to a Unix socket in daemon mode and does not listen on TCP.

mod common;

use std::{
    io,
    net::{SocketAddr, TcpListener},
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use test_helpers::*;

fn tcp_addrs_available_before_spawn() -> Vec<SocketAddr> {
    ["127.0.0.1:8080", "[::1]:8080"]
        .into_iter()
        .filter_map(|addr| addr.parse::<SocketAddr>().ok())
        .filter_map(|addr| match TcpListener::bind(addr) {
            Ok(listener) => {
                drop(listener);
                Some(addr)
            }
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => None,
            Err(e) if e.kind() == io::ErrorKind::AddrNotAvailable => None,
            Err(e) => panic!("unexpected error probing {addr}: {e}"),
        })
        .collect()
}

#[test]
fn auraed_daemon_mode_should_bind_only_unix_socket() {
    skip_if_not_root!("auraed_daemon_mode_should_bind_only_unix_socket");
    skip_if_seccomp!("auraed_daemon_mode_should_bind_only_unix_socket");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = tempdir.path().join("runtime");
    let library_dir = tempdir.path().join("library");
    std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
    std::fs::create_dir_all(&library_dir).expect("library dir");

    let tls = generate_tls_material(tempdir.path());

    let tcp_addrs = tcp_addrs_available_before_spawn();

    let child = Command::new(env!("CARGO_BIN_EXE_auraed"))
        .arg("--runtime-dir")
        .arg(runtime_dir.to_str().expect("runtime dir"))
        .arg("--library-dir")
        .arg(library_dir.to_str().expect("library dir"))
        .arg("--ca-crt")
        .arg(tls.ca_crt.to_str().expect("ca crt"))
        .arg("--server-crt")
        .arg(tls.server_crt.to_str().expect("server crt"))
        .arg("--server-key")
        .arg(tls.server_key.to_str().expect("server key"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn auraed");
    let _guard = common::ChildGuard::new(child);

    let socket_path = runtime_dir.join("aurae.sock");
    wait_for_socket(&socket_path, Duration::from_secs(5));

    let meta =
        std::fs::symlink_metadata(&socket_path).expect("metadata for socket");
    assert!(
        meta.file_type().is_socket(),
        "expected {:?} to be a Unix socket",
        socket_path
    );

    // Default daemon mode should not open the documented TCP endpoint ([::1]:8080 or 127.0.0.1:8080).
    // Only check addresses that were free before spawning auraed to avoid false positives from other services.
    for addr in tcp_addrs {
        let tcp_result = TcpListener::bind(addr).map(|listener| drop(listener));
        assert!(
            tcp_result.is_ok(),
            "expected no TCP listener at {addr}, but binding failed after starting auraed"
        );
    }
}

fn wait_for_socket(path: &Path, timeout: Duration) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("socket {path:?} not created within {:?}", timeout);
}

struct TlsMaterial {
    ca_crt: PathBuf,
    server_crt: PathBuf,
    server_key: PathBuf,
}

fn generate_tls_material(dir: &Path) -> TlsMaterial {
    let ca_crt = dir.join("ca.crt");
    let ca_key = dir.join("ca.key");
    let server_csr = dir.join("server.csr");
    let server_crt = dir.join("server.crt");
    let server_key = dir.join("server.key");

    Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-nodes",
            "-newkey",
            "rsa:2048",
            "-sha256",
            "-days",
            "365",
            "-keyout",
            ca_key.to_str().unwrap(),
            "-out",
            ca_crt.to_str().unwrap(),
            "-subj",
            "/CN=AuraeTestCA",
        ])
        .status()
        .expect("run openssl for CA")
        .success()
        .then_some(())
        .expect("openssl CA generation failed");

    Command::new("openssl")
        .args([
            "req",
            "-nodes",
            "-newkey",
            "rsa:2048",
            "-keyout",
            server_key.to_str().unwrap(),
            "-out",
            server_csr.to_str().unwrap(),
            "-subj",
            "/CN=server.unsafe.aurae.io",
        ])
        .status()
        .expect("run openssl for server csr")
        .success()
        .then_some(())
        .expect("openssl server csr failed");

    Command::new("openssl")
        .args([
            "x509",
            "-req",
            "-in",
            server_csr.to_str().unwrap(),
            "-CA",
            ca_crt.to_str().unwrap(),
            "-CAkey",
            ca_key.to_str().unwrap(),
            "-CAcreateserial",
            "-out",
            server_crt.to_str().unwrap(),
            "-days",
            "365",
            "-sha256",
        ])
        .status()
        .expect("run openssl to sign server cert")
        .success()
        .then_some(())
        .expect("openssl sign server cert failed");

    TlsMaterial { ca_crt, server_crt, server_key }
}
