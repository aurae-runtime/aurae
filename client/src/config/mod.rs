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

//! Configuration used to authenticate with a remote Aurae daemon.
//!
//! [`AuraeConfig::try_default()`] follows an ordered priority for searching for
//! configuration on a client's machine.
//!
//! 1. ${HOME}/.aurae/config
//! 2. /etc/aurae/config
//! 3. /var/lib/aurae/config

pub use self::{
    auth_config::AuthConfig, cert_material::CertMaterial,
    client_cert_details::ClientCertDetails, system_config::SystemConfig,
};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use x509_details::X509Details;

mod auth_config;
mod cert_material;
mod client_cert_details;
mod system_config;
mod x509_details;

/// Configuration for AuraeScript client
#[derive(Debug, Clone, Deserialize)]
pub struct AuraeConfig {
    /// Authentication material
    pub auth: AuthConfig,
    /// System configuration
    pub system: SystemConfig,
}

impl AuraeConfig {
    /// Attempt to easy-load Aurae configuration from well-known locations.
    pub fn try_default() -> Result<Self> {
        let home = std::env::var("HOME")
            .expect("missing $HOME environmental variable");

        let search_paths = [
            &format!("{home}/.aurae/config"),
            "/etc/aurae/config",
            "/var/lib/aurae/config",
        ];

        for path in search_paths {
            match Self::parse_from_file(path) {
                Ok(config) => {
                    return Ok(config);
                }
                Err(e) => {
                    eprintln!("warning: failed to parse config at {path}: {e}");
                    continue;
                }
            }
        }

        Err(anyhow!("unable to find valid config file"))
    }

    /// Attempt to parse a config file into memory.
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<AuraeConfig> {
        let mut config_toml = String::new();
        let mut file = File::open(path)?;

        if file
            .read_to_string(&mut config_toml)
            .with_context(|| "could not read AuraeConfig toml")?
            == 0
        {
            return Err(anyhow!("empty config"));
        }

        Ok(toml::from_str(&config_toml)?)
    }

    /// Create a new AuraeConfig from given options
    ///
    /// # Arguments
    ///
    /// * `ca_crt` - Path to ca cert
    /// * `client_crt` - Path to client cert
    /// * `client_key` - Path to client key
    /// * `socket` - Address to auraed
    ///
    /// Note: A new client is required for every independent execution of this process.
    pub fn from_options<S1: Into<String>, S2: Into<String>, S3: Into<String>, S4: Into<String>>(
        ca_crt: S1,
        client_crt: S2,
        client_key: S3,
        socket: S4,
    ) -> Self {
        let (ca_crt, client_crt, client_key, socket) = (
            ca_crt.into(),
            client_crt.into(),
            client_key.into(),
            socket.into(),
        );
        let auth = AuthConfig { ca_crt, client_crt, client_key };
        let system = SystemConfig { socket };
        Self { auth, system }
    }
}
