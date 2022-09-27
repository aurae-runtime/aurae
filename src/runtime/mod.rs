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

// use crate::codes::*;
// use crate::meta;
// use crate::new_client;
//use crate::runtime::runtime_client::RuntimeClient;

//use std::process;

#[derive(Debug, Clone)]
pub struct Runtime {}

// Register
// Start
// Stop
// Destroy

impl RegisterExecutableResponse {
    pub fn raw(&mut self) {
        println!("{:?}", self);
    }

    pub fn json(&mut self) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", serialized);
    }
}

impl StartExecutableResponse {
    pub fn raw(&mut self) {
        println!("{:?}", self);
    }

    pub fn json(&mut self) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", serialized);
    }
}

impl StopExecutableResponse {
    pub fn raw(&mut self) {
        println!("{:?}", self);
    }

    pub fn json(&mut self) {
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", serialized);
    }
}

impl DestroyExecutableResponse {
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
    // pub fn status(&mut self) -> StatusResponse {
    //     match tokio::runtime::Runtime::new() {
    //         Ok(rt) => {
    //             let client = rt.block_on(new_client());
    //             match client {
    //                 Ok(ch) => {
    //                     let mut client = RuntimeClient::new(ch.channel);
    //                     let mut meta = Vec::new();
    //                     meta.push(meta::AuraeMeta {
    //                         code: 0,
    //                         message: "".into(),
    //                     });
    //                     let request =
    //                         tonic::Request::new(StatusRequest { meta });
    //                     let res = rt.block_on(client.status(request));
    //                     match res {
    //                         Ok(status) => {
    //                             let obj = status.into_inner();
    //                             obj // Return StatusResponse
    //                         }
    //                         Err(e) => {
    //                             eprintln!("Unable to get status: {:?}", e);
    //                             process::exit(EXIT_REQUEST_FAILURE);
    //                         }
    //                     }
    //                 }
    //                 Err(e) => {
    //                     eprintln!("Unable to get status: {:?}", e);
    //                     process::exit(EXIT_CONNECT_FAILURE);
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             eprintln!("Unable to get status: {:?}", e);
    //             process::exit(EXIT_RUNTIME_ERROR);
    //         }
    //     }
    // }
}
