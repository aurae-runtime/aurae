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

//! Configuration used to authenticate with a remote Aurae daemon.
//!
//! Follows an ordered priority for searching for configuration on a clients
//! machine.
//!
//! 1. ${HOME}/.aurae/config
//! 2. /etc/aurae/config
//! 3. /var/lib/aurae/config

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use toml;

/// Configuration for AuraeScript client
#[derive(Debug, Clone, Deserialize)]
pub struct AuraeConfig {
    /// Authentication material
    pub auth: Auth,
    /// System configuration
    pub system: System,
}

/// The system configuration for AuraeScript.
///
/// Used to define settings for AuraeScript at runtime.
#[derive(Debug, Clone, Deserialize)]
pub struct System {
    /// Socket to connect the client to.
    pub socket: String,
}

/// Authentication material for an AuraeScript client.
///
/// This material is read from disk many times during runtime.
/// Changing this material during a process will impact the currently
/// running process.
#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    /// The same CA certificate the server has.
    pub ca_crt: String,
    /// The unique client certificate signed by the server.
    pub client_crt: String,
    /// The client secret key.
    pub client_key: String,
}

/// Attempt to easy-load Aurae configuration from well-known locations.
pub fn default_config() -> Result<AuraeConfig> {
    let home =
        std::env::var("HOME").expect("missing $HOME environmental variable");
    let path = format!("{}/.aurae/config", home);
    let res = parse_aurae_config(path);
    if res.is_ok() {
        return res;
    }

    let res = parse_aurae_config("/etc/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    let res = parse_aurae_config("/var/lib/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    Err(anyhow!("unable to find config file"))
}

/// Attempt to parse a config file into memory.
pub fn parse_aurae_config(path: String) -> Result<AuraeConfig> {
    let mut config_toml = String::new();
    let mut file = File::open(&path)?;

    if file.read_to_string(&mut config_toml)
           .with_context(|| "could not read AuraeConfig toml")? == 0 {
        return Err(anyhow!("empty config"));
    }

    Ok(toml::from_str(&config_toml)?)
}
