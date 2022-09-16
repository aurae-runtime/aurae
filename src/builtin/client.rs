/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */
#![allow(unused_imports)]

use crate::config::*;
use crate::observe::*;
use crate::runtime::*;
use anyhow::{Context, Result};
use std::os::unix::net::SocketAddr;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use x509_certificate::certificate::*;

#[derive(Debug, Clone)]
pub struct AuraeClient {
    pub channel: Option<Channel>,
    certmaterial: Option<Vec<u8>>,
}

// We must define a known address that will under no circumstance resolve.
// This is intended to never be used, however in the off-chance some code
// tries to leverage this later, we set it to a known faulty value.
const KNOWN_IGNORED_SOCKET_ADDR: &str = "hxxp://null";

impl AuraeClient {
    pub fn new() -> Self {
        Self {
            channel: None,
            certmaterial: None,
        }
    }
    async fn client_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = default_config()?;

        let server_root_ca_cert = tokio::fs::read(res.auth.ca_crt)
            .await
            .with_context(|| "could not read ca crt")?;

        let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);

        let client_cert = tokio::fs::read(res.auth.client_crt.clone())
            .await
            .with_context(|| "could not read client crt")?;

        let client_key = tokio::fs::read(&res.auth.client_key)
            .await
            .with_context(|| "could not read client key")?;

        let client_identity = Identity::from_pem(client_cert.clone(), client_key.clone());

        let tls = ClientTlsConfig::new()
            .domain_name("server.unsafe.aurae.io")
            .ca_certificate(server_root_ca_cert)
            .identity(client_identity);

        let channel = Channel::from_static(KNOWN_IGNORED_SOCKET_ADDR)
            .tls_config(tls)?
            .connect_with_connector(service_fn(move |_: Uri| {
                // Connect to a Uds socket
                UnixStream::connect(res.system.socket.clone())
            }))
            .await
            .with_context(|| "unable to connect auraed system socket")?;

        // We are connected, cache our details
        self.channel = Some(channel);

        // Move the raw material to the heap
        self.certmaterial = Some(client_cert.clone());
        Ok(())
    }
    pub fn runtime(&mut self) -> Runtime {
        Runtime {}
    }
    pub fn observe(&mut self) -> Observe {
        Observe {}
    }

    pub fn info(&mut self) {
        if let Some(cm) = &self.certmaterial {
            let res = X509Certificate::from_pem(cm);
            match res {
                Ok(info) => {
                    println!("Identity Name : {}", info.subject_common_name().unwrap());
                    println!("Issuer Name   : {}", info.issuer_common_name().unwrap());
                    println!("Fingerprint   : {:?}", info.sha256_fingerprint().unwrap());
                    println!("Key Algorithm : {}", info.key_algorithm().unwrap());
                }
                _ => println!("DISCONNECTED: unable to parse x509"),
            }
        } else {
            println!("DISCONNECTED: missing cert material");
        }
    }
}

use std::process;

const EXIT_CONNECT_FAILURE: i32 = 1;

pub fn connect() -> AuraeClient {
    let mut client = AuraeClient {
        channel: None,
        certmaterial: None,
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(client.client_connect());
    if let Err(e) = result {
        eprintln!("Unable to connect to Auraed: {:?}", e);
        process::exit(EXIT_CONNECT_FAILURE);
    }
    client
}
