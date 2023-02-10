/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

//! An internally scoped rust client specific for Auraed & AuraeScript.
//!
//! Manages authenticating with remote Aurae instances, as well as searching
//! the local filesystem for configuration and authentication material.

use crate::config::{AuraeConfig, CertMaterial, ClientCertDetails};
use std::str::FromStr;
use thiserror::Error;
use tokio::net::UnixStream;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, Uri};
use tower::service_fn;

const KNOWN_IGNORED_SOCKET_ADDR: &str = "hxxp://null";

type Result<T> = std::result::Result<T, ClientError>;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    ConnectionError(#[from] tonic::transport::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Instance of a single client for an Aurae consumer.
#[derive(Debug, Clone)]
pub struct Client {
    /// The channel used for gRPC connections before encryption is handled.
    pub(crate) channel: Channel,
    #[allow(unused)]
    client_cert_details: ClientCertDetails,
}

impl Client {
    pub async fn default() -> Result<Self> {
        Self::new(AuraeConfig::try_default()?).await
    }

    /// Create a new Client.
    ///
    /// Note: A new client is required for every independent execution of this process.
    pub async fn new(
        AuraeConfig { auth, system }: AuraeConfig,
    ) -> Result<Self> {
        let cert_material = auth.to_cert_material().await?;
        let client_cert_details = cert_material.get_client_cert_details()?;

        let CertMaterial { server_root_ca_cert, client_cert, client_key } =
            cert_material;

        let tls_config = ClientTlsConfig::new()
            .domain_name("server.unsafe.aurae.io") // TODO: Does this need to be configurable?
            .ca_certificate(Certificate::from_pem(server_root_ca_cert))
            .identity(Identity::from_pem(client_cert, client_key));

        // If the system socket looks like a URI, bind to it directly.  Otherwise, connect as a
        // UNIX socket (assume it's a file path).
        let channel = if let Ok(uri) = url::Url::parse(&system.socket) {
            let uri = Uri::from_str(uri.as_str()).expect("valid uri");
            Channel::builder(uri).tls_config(tls_config)?.connect().await
        } else {
            let socket = system.socket.clone();
            Channel::from_static(KNOWN_IGNORED_SOCKET_ADDR)
                .tls_config(tls_config)?
                .connect_with_connector(service_fn(move |_: Uri| {
                    UnixStream::connect(socket.clone())
                }))
                .await
        }?;

        Ok(Self { channel, client_cert_details })
    }
}
