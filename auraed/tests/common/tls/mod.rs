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
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TlsMaterial {
    #[allow(dead_code)]
    pub ca_crt: PathBuf,
    #[allow(dead_code)]
    pub server_crt: PathBuf,
    #[allow(dead_code)]
    pub server_key: PathBuf,
    #[allow(dead_code)]
    pub client_crt: Option<PathBuf>,
    #[allow(dead_code)]
    pub client_key: Option<PathBuf>,
}

#[allow(dead_code)]
pub fn generate_server_tls(dir: &Path) -> TlsMaterial {
    let ca_crt = dir.join("ca.crt");
    let ca_key = dir.join("ca.key");

    run_openssl(Command::new("openssl").args([
        "req",
        "-quiet",
        "-batch",
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
    ]));

    let server_key = dir.join("server.key");
    let server_csr = dir.join("server.csr");
    run_openssl(Command::new("openssl").args([
        "req",
        "-quiet",
        "-batch",
        "-nodes",
        "-newkey",
        "rsa:2048",
        "-keyout",
        server_key.to_str().unwrap(),
        "-out",
        server_csr.to_str().unwrap(),
        "-subj",
        "/CN=server.unsafe.aurae.io",
    ]));

    let server_crt = dir.join("_signed.server.crt");
    run_openssl(Command::new("openssl").args([
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
    ]));

    TlsMaterial {
        ca_crt,
        server_crt,
        server_key,
        client_crt: None,
        client_key: None,
    }
}

#[allow(dead_code)]
pub fn generate_server_and_client_tls(dir: &Path) -> TlsMaterial {
    let ca_crt = dir.join("ca.crt");
    let ca_key = dir.join("ca.key");
    run_openssl(Command::new("openssl").args([
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
    ]));

    let server_key = dir.join("server.key");
    let server_csr = dir.join("server.csr");
    run_openssl(Command::new("openssl").args([
        "req",
        "-new",
        "-newkey",
        "rsa:2048",
        "-nodes",
        "-keyout",
        server_key.to_str().unwrap(),
        "-out",
        server_csr.to_str().unwrap(),
        "-subj",
        "/CN=server.unsafe.aurae.io",
        "-addext",
        "subjectAltName = DNS:server.unsafe.aurae.io",
    ]));

    let server_crt = dir.join("_signed.server.crt");
    let server_ext = dir.join("server.ext");
    std::fs::write(
        &server_ext,
        "subjectAltName = DNS:server.unsafe.aurae.io\nextendedKeyUsage = serverAuth\n",
    )
    .expect("write server ext");
    run_openssl(Command::new("openssl").args([
        "x509",
        "-req",
        "-days",
        "365",
        "-in",
        server_csr.to_str().unwrap(),
        "-CA",
        ca_crt.to_str().unwrap(),
        "-CAkey",
        ca_key.to_str().unwrap(),
        "-CAcreateserial",
        "-out",
        server_crt.to_str().unwrap(),
        "-extfile",
        server_ext.to_str().unwrap(),
    ]));

    let client_key = dir.join("client.key");
    let client_csr = dir.join("client.csr");
    run_openssl(Command::new("openssl").args([
        "req",
        "-new",
        "-newkey",
        "rsa:2048",
        "-nodes",
        "-keyout",
        client_key.to_str().unwrap(),
        "-out",
        client_csr.to_str().unwrap(),
        "-subj",
        "/CN=client.unsafe.aurae.io",
        "-addext",
        "subjectAltName = DNS:client.unsafe.aurae.io",
    ]));

    let client_crt = dir.join("_signed.client.crt");
    let client_ext = dir.join("client.ext");
    std::fs::write(
        &client_ext,
        "subjectAltName = DNS:client.unsafe.aurae.io\nextendedKeyUsage = clientAuth\n",
    )
    .expect("write client ext");
    run_openssl(Command::new("openssl").args([
        "x509",
        "-req",
        "-days",
        "365",
        "-in",
        client_csr.to_str().unwrap(),
        "-CA",
        ca_crt.to_str().unwrap(),
        "-CAkey",
        ca_key.to_str().unwrap(),
        "-CAcreateserial",
        "-out",
        client_crt.to_str().unwrap(),
        "-extfile",
        client_ext.to_str().unwrap(),
    ]));

    TlsMaterial {
        ca_crt,
        server_crt,
        server_key,
        client_crt: Some(client_crt),
        client_key: Some(client_key),
    }
}

fn run_openssl(cmd: &mut Command) {
    let status = cmd.status().expect("failed to run openssl");
    if !status.success() {
        panic!("openssl command {:?} failed with status {status:?}", cmd);
    }
}