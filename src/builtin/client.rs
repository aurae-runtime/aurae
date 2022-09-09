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

use crate::config::*;
use crate::observe::*;
use crate::runtime::*;
use std::os::unix::net::SocketAddr;
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

#[derive(Debug, Clone)]
pub struct AuraeClient {}

impl AuraeClient {
    pub fn new() -> Self {
        Self {}
    }
    async fn client_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = default_config()?;

        // TODO @kris-nova
        // TODO We need to populate the AuraeClient{} with connection details so aurae.info(); works
        let server_root_ca_cert = tokio::fs::read(res.auth.ca_crt).await?;
        let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
        let client_cert = tokio::fs::read(res.auth.client_crt).await?;
        let client_key = tokio::fs::read(res.auth.client_key).await?;
        let client_identity = Identity::from_pem(client_cert, client_key);

        let tls = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(server_root_ca_cert)
            .identity(client_identity);

        // Aurae leverages Unix Abstract Sockets
        // Read more about Abstract Sockets: https://man7.org/linux/man-pages/man7/unix.7.html
        // TODO Consider this: https://docs.rs/nix/latest/nix/sys/socket/struct.UnixAddr.html#method.new_abstract

        let addr = SocketAddr::from_abstract_namespace(b"aurae")?; // Linux only
        let stream = std::os::unix::net::UnixStream::connect_addr(&addr)?;

        // TODO We need to call Unix domain socket: https://github.com/hyperium/tonic/blob/master/examples/src/uds/client.rs
        // TODO we need to pass "tls" to the unix domain socket connection

        let channel = Endpoint::try_from("")?
            .tls_config(tls)?
            .connect_with_connector(service_fn(|_: Uri| UnixStream::from_std(stream)))
            .await?;

        Ok(())
    }
    pub fn runtime(&mut self) -> Runtime {
        Runtime {}
    }
    pub fn observe(&mut self) -> Observe {
        Observe {}
    }
    pub fn info(&mut self) {
        println!("Connection details")
    }
}

pub fn connect() -> AuraeClient {
    let mut client = AuraeClient {};
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(client.client_connect());
    if let Err(e) = result {
        eprintln!("Unable to connect: {:?}", e)
    }
    client
}
