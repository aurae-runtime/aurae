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

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;

/// The system configuration for AuraeScript.
///
/// Used to define settings for AuraeScript at runtime.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    /// Socket to connect the client to.  Can be a path (unix socket) or a network socket address.
    ///
    /// When deserializing from a string, the deserializer will try to parse a valid value in the following order:
    /// - IpV6 with scope id (e.g., "[fe80::2%4]:8080")
    /// - IpV6 without scope id (e.g., "[fe80::2]:8080")
    /// - IpV4 (e.g., "127.0.0.1:8080")
    /// - Otherwise a path
    ///
    /// scope id must be a valid u32, otherwise it will be assumed a path
    pub socket: AuraeSocket,
}

#[derive(Debug, Clone)]
pub enum AuraeSocket {
    Path(PathBuf),
    Addr(SocketAddr),
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
        if let Ok(addr) = v.parse::<SocketAddrV6>() {
            Ok(AuraeSocket::Addr(addr.into()))
        } else if let Ok(addr) = v.parse::<SocketAddrV4>() {
            Ok(AuraeSocket::Addr(addr.into()))
        } else {
            Ok(AuraeSocket::Path(v.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};
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

        let AuraeSocket::Addr (addr) = res else {
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
    fn can_parse_aurae_socket_ipv6_with_scope_id() {
        let visitor = AuraeSocketVisitor {};

        let res =
            visitor.visit_str::<toml::de::Error>("[fe80::2%4]:8080").unwrap();

        let AuraeSocket::Addr (addr) = res else {
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
    fn can_parse_aurae_socket_ipv4() {
        let visitor = AuraeSocketVisitor {};

        let res =
            visitor.visit_str::<toml::de::Error>("127.0.0.1:8081").unwrap();

        let AuraeSocket::Addr (addr) = res else {
            panic!("expected AuraeSocket::Addr");
        };

        let SocketAddr::V4(addr) = addr else {
            panic!("expected v4 addr");
        };

        assert_eq!(*addr.ip(), Ipv4Addr::from_str("127.0.0.1").unwrap());
        assert_eq!(addr.port(), 8081);
    }
}