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
tonic::include_proto!("meta");

use crate::codes::*;
use crate::new_client;
use crate::runtime::runtime_client::RuntimeClient;

use std::process;

pub fn exec() -> Executable {
    Executable::default()
}

impl Executable {
    pub fn get_comment(&mut self) -> String {
        self.comment.clone()
    }

    pub fn set_comment(&mut self, x: String) {
        self.comment = x;
    }
    pub fn get_exec(&mut self) -> String {
        self.exec.clone()
    }

    pub fn set_exec(&mut self, x: String) {
        self.exec = x;
    }

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, x: String) {
        self.name = x;
    }

    pub fn raw(&mut self) {
        println!("{:?}", self);
    }

    pub fn json(&mut self) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", serialized);
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutableStatus {
    pub fn raw(&mut self) {
        println!("{:?}", self);
    }

    pub fn json(&mut self) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", serialized);
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }
    pub fn register_executable(&mut self, req: Executable) -> ExecutableStatus {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client = RuntimeClient::new(ch.channel);
                        let res = rt.block_on(client.register_executable(req));
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

    pub fn start_executable(&mut self, req: Executable) -> ExecutableStatus {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client = RuntimeClient::new(ch.channel);
                        let res = rt.block_on(client.start_executable(req));
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

    pub fn stop_executable(&mut self, req: Executable) -> ExecutableStatus {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client = RuntimeClient::new(ch.channel);
                        let res = rt.block_on(client.stop_executable(req));
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

    pub fn destroy_executable(&mut self, req: Executable) -> ExecutableStatus {
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let client = rt.block_on(new_client());
                match client {
                    Ok(ch) => {
                        let mut client = RuntimeClient::new(ch.channel);
                        let res = rt.block_on(client.destroy_executable(req));
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
