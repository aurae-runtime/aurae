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

use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
// use std::path::PathBuf;
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

pub fn default_config() -> Result<AuraeConfig, Box<dyn Error>> {
    // ${HOME}/.aura/default.config.toml
    let home = std::env::var("HOME").unwrap();
    let path = format!("{}/.aurae/config", home);
    //println!("Checking: {}", path);
    let res = parse_aurae_config(path);
    if res.is_ok() {
        return res;
    }

    // /etc/aurae/default.config.toml
    //println!("Checking: {}", "/etc/aurae/config");
    let res = parse_aurae_config("/etc/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    // /var/lib/aurae/default.config.toml
    let res = parse_aurae_config("/var/lib/aurae/config".into());
    if res.is_ok() {
        return res;
    }

    Err("Unable to load default AuraeConfig".into())
}

pub fn parse_aurae_config(path: String) -> Result<AuraeConfig, Box<dyn Error>> {
    let mut config_toml = String::new();
    let mut file = File::open(&path)?;

    file.read_to_string(&mut config_toml)?;

    Ok(toml::from_str(&config_toml)?)
}
