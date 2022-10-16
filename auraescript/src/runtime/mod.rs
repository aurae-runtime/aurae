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

tonic::include_proto!("runtime");

use crate::codes::*;
use crate::new_client;
use crate::runtime::runtime_client::RuntimeClient;

use std::process;

/// cmd will create a new Executable{} type and set the command to the input
pub fn cmd(cmd: &str) -> Executable {
    Executable { command: cmd.to_string(), ..Executable::default() }
}

/// exec() is a system alias that will create a new Executable{} type and start it
pub fn exec(x: &str) -> ExecutableStatus {
    Runtime::new().exec(cmd(x))
}

#[derive(Debug, Clone)]
pub struct Runtime {}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn exec(&mut self, req: Executable) -> ExecutableStatus {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client = RuntimeClient::new(ch.channel);
                        let res = rt.block_on(client.exec(req));
                        match res {
                            Ok(x) => x.into_inner(),
                            Err(e) => {
                                eprintln!("{:?}", e);
                                process::exit(EXIT_REQUEST_FAILURE);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
                        process::exit(EXIT_CONNECT_FAILURE);
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                process::exit(EXIT_RUNTIME_ERROR);
            }
        }
    }

    // pub fn executable_stop(&mut self, req: Executable) -> ExecutableStatus {
    //     match tokio::runtime::Runtime::new() {
    //         Ok(rt) => {
    //             let client = rt.block_on(new_client());
    //             match client {
    //                 Ok(ch) => {
    //                     let mut client = RuntimeClient::new(ch.channel);
    //                     let res = rt.block_on(client.executable_stop(req));
    //                     match res {
    //                         Ok(x) => x.into_inner(),
    //                         Err(e) => {
    //                             eprintln!("{:?}", e);
    //                             process::exit(EXIT_REQUEST_FAILURE);
    //                         }
    //                     }
    //                 }
    //                 Err(e) => {
    //                     eprintln!("{:?}", e);
    //                     process::exit(EXIT_CONNECT_FAILURE);
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             eprintln!("{:?}", e);
    //             process::exit(EXIT_RUNTIME_ERROR);
    //         }
    //     }
    // }
}
