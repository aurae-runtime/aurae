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

use crate::execute;
use aurae_client::runtime::cell_service::CellServiceClient;
use aurae_proto::runtime::{
    Cell, CellServiceAllocateRequest, CellServiceFreeRequest,
    CellServiceStartRequest, CellServiceStopRequest, CpuController,
    CpusetController, Executable,
};
use clap::Subcommand;

// We'd like to macro this struct and the impl from the CellService in the proto.
// I'm (future-highway) not sure how, even though this seems almost entirely boilerplate.
//
// Maybe macros is the wrong approach.
// Isn't there a way to customize/extend the proto. How much work it that? Is that worth it?
//
// We've lost all the documentation that is in the proto.
#[derive(Debug, Subcommand)]
pub enum CellServiceCommands {
    #[command(arg_required_else_help = true)]
    Allocate {
        #[arg(required = true)]
        cell_name: String,
        #[arg(long, alias = "cpu-weight")]
        cell_cpu_weight: Option<u64>,
        #[arg(long, alias = "cpu-max")]
        cell_cpu_max: Option<i64>,
        #[arg(long, alias = "cpuset-cpus")]
        cell_cpuset_cpus: Option<String>,
        #[arg(long, alias = "cpuset-mems")]
        cell_cpuset_mems: Option<String>,
        #[arg(long, default_value = "false")]
        cell_isolate_process: bool,
        #[arg(long, default_value = "false")]
        cell_isolate_network: bool,
    },
    #[command(arg_required_else_help = true)]
    Free {
        #[arg(required = true)]
        cell_name: String,
    },
    #[command(arg_required_else_help = true)]
    Start {
        #[arg(required = true)]
        cell_name: String,
        #[arg(required = true)]
        executable_name: String,
        #[arg(required = true, long, aliases = ["command", "cmd"], short = 'c')]
        executable_command: String,
        #[arg(long, aliases = ["description", "desc"])]
        executable_description: Option<String>,
    },
    #[command(arg_required_else_help = true)]
    Stop {
        #[arg(required = true)]
        cell_name: String,
        #[arg(required = true)]
        executable_name: String,
    },
}

impl CellServiceCommands {
    pub async fn execute(self) -> anyhow::Result<()> {
        match self {
            CellServiceCommands::Allocate {
                cell_name,
                cell_cpu_weight,
                cell_cpu_max,
                cell_cpuset_cpus,
                cell_cpuset_mems,
                cell_isolate_process,
                cell_isolate_network,
            } => {
                let req = CellServiceAllocateRequest {
                    cell: Some(Cell {
                        name: cell_name,
                        cpu: Some(CpuController {
                            weight: cell_cpu_weight,
                            max: cell_cpu_max,
                        }),
                        cpuset: Some(CpusetController {
                            cpus: cell_cpuset_cpus,
                            mems: cell_cpuset_mems,
                        }),
                        isolate_process: cell_isolate_process,
                        isolate_network: cell_isolate_network,
                    }),
                };

                let _ = execute!(CellServiceClient::allocate, req);
            }
            CellServiceCommands::Free { cell_name } => {
                let req = CellServiceFreeRequest { cell_name };
                let _ = execute!(CellServiceClient::free, req);
            }
            CellServiceCommands::Start {
                cell_name,
                executable_name,
                executable_command,
                executable_description,
            } => {
                let req = CellServiceStartRequest {
                    cell_name: Some(cell_name),
                    executable: Some(Executable {
                        name: executable_name,
                        command: executable_command,
                        description: executable_description
                            .unwrap_or_else(|| "".into()),
                    }),
                };

                let _ = execute!(CellServiceClient::start, req);
            }
            CellServiceCommands::Stop { cell_name, executable_name } => {
                let req = CellServiceStopRequest {
                    cell_name: Some(cell_name),
                    executable_name,
                };
                let _ = execute!(CellServiceClient::stop, req);
            }
        }

        Ok(())
    }
}
