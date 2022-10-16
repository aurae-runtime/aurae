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

tonic::include_proto!("schedule");

use crate::codes::*;
use crate::new_client;
use crate::schedule::schedule_executable_client::ScheduleExecutableClient;
use crate::Executable;

use std::process;

#[derive(Debug, Clone)]
pub struct ScheduleExecutable {}

impl Default for ScheduleExecutable {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleExecutable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn enable(&mut self, req: Executable) -> ExecutableEnableResponse {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client =
                            ScheduleExecutableClient::new(ch.channel);
                        let res = rt.block_on(client.enable(req));
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

    pub fn disable(&mut self, req: Executable) -> ExecutableDisableResponse {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client =
                            ScheduleExecutableClient::new(ch.channel);
                        let res = rt.block_on(client.disable(req));
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

    pub fn destroy(&mut self, req: Executable) -> ExecutableDestroyResponse {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client =
                            ScheduleExecutableClient::new(ch.channel);
                        let res = rt.block_on(client.destroy(req));
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
}
