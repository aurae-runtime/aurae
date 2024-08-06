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
    client_cert_details::ClientCertDetails, system_config::AuraeSocket,
    system_config::SystemConfig,
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
            match Self::parse_from_toml_file(path) {
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
    pub fn parse_from_toml_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<AuraeConfig> {
        let mut config_toml = String::new();
        let mut file = File::open(path)?;

        if file
            .read_to_string(&mut config_toml)
            .with_context(|| "could not read AuraeConfig toml")?
            == 0
        {
            return Err(anyhow!("empty config"));
        }

        AuraeConfig::parse_from_toml(&config_toml)
    }

    pub fn parse_from_toml(config_toml: &str) -> Result<AuraeConfig> {
        Ok(toml::from_str(config_toml)?)
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
    pub fn from_options<
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    >(
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
        let system = SystemConfig { socket: AuraeSocket::Path(socket.into()) };
        Self { auth, system }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::str::FromStr;

    fn get_input(socket: &str) -> String {
        const INPUT: &str = r#"
[auth]
ca_crt = "~/.aurae/pki/ca.crt"
client_crt = "~/.aurae/pki/_signed.client.nova.crt"
client_key = "~/.aurae/pki/client.nova.key"

[system]
socket = "#;

        format!("{INPUT}\"{socket}\"")
    }

    #[test]
    fn can_parse_toml_config_socket_path() {
        let input = get_input("/var/run/aurae/aurae.sock");
        let config = AuraeConfig::parse_from_toml(&input).unwrap();
        assert!(
            matches!(config.system.socket, AuraeSocket::Path(path) if Some("/var/run/aurae/aurae.sock") == path.to_str())
        )
    }

    #[test]
    fn can_parse_toml_config_socket_ipv6_with_scope_id() {
        let input = get_input("[fe80::2%4]:8080");
        let config = AuraeConfig::parse_from_toml(&input).unwrap();
        let AuraeSocket::Addr (addr) = config.system.socket else {
            panic!("expected AuraeSocket::Addr");
        };

        let SocketAddr::V6(addr) = addr else {
            panic!("expected v6 addr");
        };

        assert_eq!(*addr.ip(), Ipv6Addr::from_str("fe80::2").unwrap());
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.scope_id(), 4);
    }

    #[test]
    fn can_parse_toml_config_socket_ipv6_without_scope_id() {
        let input = get_input("[fe80::2]:8080");
        let config = AuraeConfig::parse_from_toml(&input).unwrap();
        let AuraeSocket::Addr (addr) = config.system.socket else {
            panic!("expected AuraeSocket::Addr");
        };

        let SocketAddr::V6(addr) = addr else {
            panic!("expected v6 addr");
        };

        assert_eq!(*addr.ip(), Ipv6Addr::from_str("fe80::2").unwrap());
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.scope_id(), 0);
    }

    #[test]
    fn can_parse_toml_config_socket_ipv4() {
        let input = get_input("127.1.2.3:1234");
        let config = AuraeConfig::parse_from_toml(&input).unwrap();
        let AuraeSocket::Addr (addr) = config.system.socket else {
            panic!("expected AuraeSocket::Addr");
        };

        let SocketAddr::V4(addr) = addr else {
            panic!("expected v4 addr");
        };

        assert_eq!(*addr.ip(), Ipv4Addr::from_str("127.1.2.3").unwrap());
        assert_eq!(addr.port(), 1234);
    }
}