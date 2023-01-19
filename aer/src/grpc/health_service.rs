/* -------------------------------------------------------------------------- *\
 *          Apache 2.0 License Copyright © 2022-2023 The Aurae Authors        *
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

use crate::{execute, execute_server_streaming};
// TODO: eliminate the double health after cri branch is merged
use aurae_client::grpc::health::health::HealthClient;
use aurae_proto::grpc::health::HealthCheckRequest;
use clap::Subcommand;
use futures_util::StreamExt;

#[derive(Debug, Subcommand)]
pub enum HealthServiceCommands {
    #[command()]
    Check {
        #[arg(long, short = 's')]
        service: Option<String>,
    },
    #[command()]
    Watch {
        #[arg(long, short = 's')]
        service: Option<String>,
    },
}

impl HealthServiceCommands {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            HealthServiceCommands::Check { service } => {
                let req = HealthCheckRequest {
                    service: service.unwrap_or_else(|| "".into()),
                };

                let _ = execute!(HealthClient::check, req);
            }
            HealthServiceCommands::Watch { service } => {
                let req = HealthCheckRequest {
                    service: service.unwrap_or_else(|| "".into()),
                };

                execute_server_streaming!(HealthClient::watch, req);
            }
        }

        Ok(())
    }
}
