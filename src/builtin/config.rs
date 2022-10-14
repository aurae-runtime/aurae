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

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use toml;

#[derive(Debug, Clone, Deserialize)]
pub struct AuraeConfig {
    pub auth: Auth,
    pub system: System,
}

// #[derive(RustcEncodable, RustcDecodable, Debug)]
#[derive(Debug, Clone, Deserialize)]
pub struct System {
    pub socket: String,
}

// #[derive(RustcEncodable, RustcDecodable, Debug)]
#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    // Root CA
    pub ca_crt: String,

    pub client_crt: String,
    pub client_key: String,
}

pub fn default_config() -> Result<AuraeConfig> {
    // ${HOME}/.aura/config
    let home =
        std::env::var("HOME").expect("missing $HOME environmental variable");
    let path = format!("{}/.aurae/config", home);
    //println!("Checking: {}", path);
    let res = parse_aurae_config(path);
    if res.is_ok() {
        return res;
    }

    // /etc/aurae/config
    //println!("Checking: {}", "/etc/aurae/config");
    let res = parse_aurae_config("/etc/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    // /var/lib/aurae/config
    let res = parse_aurae_config("/var/lib/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    Err(anyhow!("unable to find config file"))
}

pub fn parse_aurae_config(path: String) -> Result<AuraeConfig> {
    let mut config_toml = String::new();
    let mut file = File::open(&path)?;

    file.read_to_string(&mut config_toml)
        .with_context(|| "could not read AuraeConfig toml")?;

    Ok(toml::from_str(&config_toml)?)
}
