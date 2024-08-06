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

use crate::config::cert_material::CertMaterial;
use serde::Deserialize;

/// Authentication material for an AuraeScript client.
///
/// This material is read from disk many times during runtime.
/// Changing this material during a process will impact the currently
/// running process.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// The same CA certificate the server has.
    pub ca_crt: String,
    /// The unique client certificate signed by the server.
    pub client_crt: String,
    /// The client secret key.
    pub client_key: String,
}

impl AuthConfig {
    pub async fn to_cert_material(&self) -> anyhow::Result<CertMaterial> {
        CertMaterial::from_config(self).await
    }
}