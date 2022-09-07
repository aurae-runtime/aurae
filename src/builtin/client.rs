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

use crate::runtime::*;
use crate::observe::*;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

#[derive(Debug, Clone)]
pub struct AuraeClient {

}

impl AuraeClient {
    pub fn new() -> Self {
        Self {}
    }
    async fn client_connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        // TODO @kris-nova
        // TODO We need to define an "Aurae config" for the client
        // TODO We need to plumb the values here back from the config
        // TODO We need to populate the AuraeClient{} with connection details so aurae.info(); works
        let server_root_ca_cert = tokio::fs::read("/etc/aurae/pki/ca.crt.pem").await?;
        let server_root_ca_cert = Certificate::from_pem(server_root_ca_cert);
        let client_cert = tokio::fs::read("/etc/aurae/pki/_signed.client.nova.crt.pem").await?;
        let client_key = tokio::fs::read("/etc/aurae/pki/client.nova.key.pem").await?;
        let client_identity = Identity::from_pem(client_cert, client_key);

        let tls = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(server_root_ca_cert)
            .identity(client_identity);

        // TODO We need to call Unix domain socket: https://github.com/hyperium/tonic/blob/master/examples/src/uds/client.rs
        let channel = Channel::from_static("http://[::1]:50051")
            .tls_config(tls)?
            .connect()
            .await?;

        Ok(())
    }
    pub fn runtime(&mut self) -> Runtime {
        Runtime{}
    }
    pub fn observe(&mut self) -> Observe {
        Observe{}
    }
    pub fn info(&mut self) {
        println!("Connection details")
    }
}

pub fn connect() -> AuraeClient {
    let mut client = AuraeClient{};
    let rt  = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(client.client_connect());
    if let Err(e) = result {
        eprintln!("{:?}", e)
    }
    client
}



