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

//! An internally scoped rust client specific for AuraeScript.
//!
//! Manages authenticating with remote Aurae instances, as well as searching
//! the local filesystem for configuration and authentication material.

use crate::builtin::config::AuraeConfig;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;
use tonic::transport::Uri;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use tower::service_fn;
use x509_certificate::X509Certificate;

const KNOWN_IGNORED_SOCKET_ADDR: &str = "hxxp://null";

/// Instance of a single client for an Aurae consumer.
#[derive(Debug, Clone, Serialize)]
pub struct AuraeClient {
    /// The channel used for gRPC connections before encryption is handled.
    #[serde(skip)]
    pub(crate) channel: Channel,
    x509_details: X509Details,
}

impl AuraeClient {
    pub async fn default() -> anyhow::Result<Self> {
        Self::new(AuraeConfig::try_default()?).await
    }

    /// Create a new AuraeClient.
    ///
    /// Note: A new client is required for every independent execution of this process.
    pub(crate) async fn new(
        AuraeConfig { auth, system }: AuraeConfig,
    ) -> anyhow::Result<Self> {
        let server_root_ca_cert = tokio::fs::read(&auth.ca_crt)
            .await
            .with_context(|| "could not read ca crt")?;

        let client_cert = tokio::fs::read(&auth.client_crt)
            .await
            .with_context(|| "could not read client crt")?;

        let client_key = tokio::fs::read(&auth.client_key)
            .await
            .with_context(|| "could not read client key")?;

        let tls_config = ClientTlsConfig::new()
            .domain_name("server.unsafe.aurae.io")
            .ca_certificate(Certificate::from_pem(server_root_ca_cert))
            .identity(Identity::from_pem(
                client_cert.clone(),
                client_key.clone(),
            ));

        let x509 = X509Certificate::from_pem(client_cert.clone())?;

        let subject_common_name = x509
            .subject_common_name()
            .ok_or_else(|| anyhow!("missing subject_common_name"))?;

        let issuer_common_name = x509
            .issuer_common_name()
            .ok_or_else(|| anyhow!("missing issuer_common_name"))?;

        let sha256_fingerprint = x509.sha256_fingerprint()?;

        let key_algorithm = x509
            .key_algorithm()
            .ok_or_else(|| anyhow!("missing key_algorithm"))?
            .to_string();

        let x509_details = X509Details {
            subject_common_name,
            issuer_common_name,
            sha256_fingerprint: format!("{:?}", sha256_fingerprint),
            key_algorithm,
        };

        let channel = Channel::from_static(KNOWN_IGNORED_SOCKET_ADDR)
            .tls_config(tls_config)?
            .connect_with_connector(service_fn(move |_: Uri| {
                UnixStream::connect(system.socket.clone())
            }))
            .await
            .with_context(|| "unable to connect auraed system socket")?;

        Ok(Self { channel, x509_details })
    }
}

/// An in-memory representation of an X509 identity, and its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X509Details {
    /// From the SSL spec, the subject common name.
    pub subject_common_name: String,
    /// From the SSL spec, the issuer common name.
    pub issuer_common_name: String,
    /// From the SSL spec, the sha256 sum fingerprint of the material.
    pub sha256_fingerprint: String,
    /// From the SSL spec, the algorithm used for encryption.
    pub key_algorithm: String,
}
