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

use aer::{
    discovery::DiscoveryServiceCommands, grpc::HealthCommands,
    observe::ObserveServiceCommands, runtime::CellServiceCommands,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Cell {
        #[command(subcommand)]
        command: CellServiceCommands,
    },
    #[command(arg_required_else_help = true)]
    Discovery {
        #[command(subcommand)]
        command: DiscoveryServiceCommands,
    },
    #[command(arg_required_else_help = true)]
    Health {
        #[command(subcommand)]
        command: HealthCommands,
    },
    #[command(arg_required_else_help = true)]
    Observe {
        #[command(subcommand)]
        command: ObserveServiceCommands,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    if let Err(e) = match args.command {
        Commands::Cell { command } => command.execute().await,
        Commands::Discovery { command } => command.execute().await,
        Commands::Health { command } => command.execute().await,
        Commands::Observe { command } => command.execute().await,
    } {
        eprintln!("{e:#?}");
    }
}