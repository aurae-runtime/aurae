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

#![warn(clippy::unwrap_used)]

use anyhow::anyhow;
use anyhow::Context;
use log::*;
use sea_orm::ConnectOptions;
use sea_orm::ConnectionTrait;
use sea_orm::Database;
use sea_orm::Statement;
use std::borrow::Cow;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

use crate::observe::observe_server::ObserveServer;
use crate::observe::ObserveService;
use crate::runtime::runtime_server::RuntimeServer;
use crate::runtime::RuntimeService;
use crate::schedule::schedule_executable_server::ScheduleExecutableServer;
use crate::schedule::ScheduleExecutableService;

pub mod init;
mod meta;
mod observe;
mod runtime;
mod schedule;

pub const AURAE_SOCK: &str = "/var/run/aurae/aurae.sock";

#[derive(Debug)]
pub struct AuraedRuntime {
    // Root CA
    pub ca_crt: PathBuf,

    pub server_crt: PathBuf,
    pub server_key: PathBuf,
    pub socket: PathBuf,
}

impl AuraedRuntime {
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Manage the socket permission/groups first\
        let _ = fs::remove_file(&self.socket);
        let sock_path = Path::new(&self.socket)
            .parent()
            .ok_or("unable to find socket path")?;
        tokio::fs::create_dir_all(sock_path).await.with_context(|| {
            format!(
                "Failed to create directory for socket: {}",
                self.socket.display()
            )
        })?;
        trace!("{:#?}", self);

        let server_crt =
            tokio::fs::read(&self.server_crt).await.with_context(|| {
                format!(
                    "Failed to read server certificate: {}",
                    self.server_crt.display()
                )
            })?;
        let server_key = tokio::fs::read(&self.server_key).await?;
        let db_key = server_key.clone();
        let server_identity = Identity::from_pem(server_crt, server_key);
        info!("Register Server SSL Identity");

        let ca_crt = tokio::fs::read(&self.ca_crt).await?;
        let ca_crt_pem = Certificate::from_pem(ca_crt.clone());

        let tls = ServerTlsConfig::new()
            .identity(server_identity)
            .client_ca_root(ca_crt_pem);

        info!("Validating SSL Identity and Root Certificate Authority (CA)");

        let sock = UnixListener::bind(&self.socket)?;
        let sock_stream = UnixListenerStream::new(sock);

        // Run the server concurrently
        let handle = tokio::spawn(async {
            Server::builder()
                .tls_config(tls)?
                .add_service(RuntimeServer::new(RuntimeService::default()))
                .add_service(ObserveServer::new(ObserveService::default()))
                .add_service(ScheduleExecutableServer::new(
                    ScheduleExecutableService::default(),
                ))
                .serve_with_incoming(sock_stream)
                .await
        });

        trace!("Setting socket mode {} -> 766", &self.socket.display());

        // We set the mode to 766 for the Unix domain socket.
        // This is what allows non-root users to dial the socket
        // and authenticate with mTLS.
        fs::set_permissions(&self.socket, fs::Permissions::from_mode(0o766))?;
        info!("User Access Socket Created: {}", self.socket.display());

        // SQLite
        info!("Database Location:  /var/lib/aurae.db");
        info!("Unlocking SQLite Database with Key: {:?}", self.server_key);
        let mut opt =
            ConnectOptions::new("sqlite:/var/lib/aurae.db".to_owned());
        opt.sqlx_logging(false).sqlcipher_key(Cow::from(format!(
            "{:?}",
            db_key.to_ascii_lowercase()
        )));

        // Pragma initial connection
        let mut opt = ConnectOptions::new("sqlite::memory:".to_owned());
        opt.sqlx_logging(false); // TODO add sqlcipher_key
        let db = Database::connect(opt).await?;
        let x = db
            .execute(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA database_list;".to_string(),
            ))
            .await?;
        info!("Initializing: SQLite: {:?}", x);

        //runtime::hydrate(&db).await?;

        // Event loop
        handle.await??;
        info!("gRPC server exited successfully");

        Ok(())
    }
}

pub fn command_from_string(cmd: &str) -> Result<Command, anyhow::Error> {
    let mut entries = cmd.split(' ');
    let base = match entries.next() {
        Some(base) => base,
        None => {
            return Err(anyhow!("empty base command string"));
        }
    };
    let mut command = Command::new(base);
    for ent in entries {
        if ent != base {
            command.arg(ent);
        }
    }
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        assert_eq!(AURAE_SOCK, "/var/run/aurae/aurae.sock");
    }
}
