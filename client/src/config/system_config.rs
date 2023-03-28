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

use if_chain::if_chain;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::net::SocketAddrV6;
use std::path::PathBuf;

/// The system configuration for AuraeScript.
///
/// Used to define settings for AuraeScript at runtime.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    /// Socket to connect the client to.  Can be a path (unix socket) or a network socket address.
    pub socket: AuraeSocket,
}

#[derive(Debug, Clone)]
pub enum AuraeSocket {
    Path(PathBuf),
    IPv6 { ip: SocketAddrV6, scope_id: Option<u32> },
}

impl<'de> Deserialize<'de> for AuraeSocket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(AuraeSocketVisitor)
    }
}

struct AuraeSocketVisitor;

impl<'de> Visitor<'de> for AuraeSocketVisitor {
    type Value = AuraeSocket;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a path (unix socket) or a network socket address")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if_chain! {
            if let Some((ip, scope_id)) = v.rsplit_once('%');
            if let Ok(ip) = ip.parse::<SocketAddrV6>();
            if let Ok(scope_id) = scope_id.parse::<u32>();
            then {
                Ok(AuraeSocket::IPv6 {ip, scope_id: Some(scope_id)})
            }
            else {
                if let Ok(ip) = v.parse::<SocketAddrV6>() {
                    Ok(AuraeSocket::IPv6 {ip, scope_id: None})
                } else {
                    Ok(AuraeSocket::Path(v.into()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn can_parse_aurae_socket_path() {
        let visitor = AuraeSocketVisitor {};

        let res = visitor
            .visit_str::<toml::de::Error>("/var/run/aurae/aurae.sock")
            .unwrap();

        assert!(
            matches!(res, AuraeSocket::Path(path) if Some("/var/run/aurae/aurae.sock") == path.to_str())
        );
    }

    #[test]
    fn can_parse_aurae_socket_ipv6() {
        let visitor = AuraeSocketVisitor {};

        let res =
            visitor.visit_str::<toml::de::Error>("[fe80::2]:8080").unwrap();

        let AuraeSocket::IPv6 {ip, scope_id} = res else {
            panic!("expected AuraeSocket::IPv6");
        };

        assert_eq!(ip, SocketAddrV6::from_str("[fe80::2]:8080").unwrap());
        assert_eq!(scope_id, None);
    }

    #[test]
    fn can_parse_aurae_socket_ipv6_with_scope_id() {
        let visitor = AuraeSocketVisitor {};

        let res =
            visitor.visit_str::<toml::de::Error>("[fe80::2]:8080%4").unwrap();

        let AuraeSocket::IPv6 {ip, scope_id} = res else {
            panic!("expected AuraeSocket::IPv6");
        };

        assert_eq!(ip, SocketAddrV6::from_str("[fe80::2]:8080").unwrap());
        assert_eq!(scope_id, Some(4));
    }
}
