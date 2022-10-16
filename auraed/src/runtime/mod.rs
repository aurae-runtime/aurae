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
/*
 * [Runtime] is a SYNCHRONOUS subsystem.
 */

#![allow(dead_code)]
tonic::include_proto!("runtime");

use crate::runtime::runtime_server::Runtime;
use crate::{command_from_string, meta};
use tonic::{Request, Response, Status};

#[derive(Debug, Default, Clone)]
pub struct RuntimeService {}

#[tonic::async_trait]
impl Runtime for RuntimeService {
    async fn exec(
        &self,
        request: Request<Executable>,
    ) -> Result<Response<ExecutableStatus>, Status> {
        let r = request.into_inner();
        let cmd = command_from_string(&r.command);
        match cmd {
            Ok(mut cmd) => {
                let output = cmd.output();
                match output {
                    Ok(output) => {
                        let meta = meta::AuraeMeta {
                            name: r.command,
                            message: "-".to_string(),
                        };
                        let proc = meta::ProcessMeta { pid: -1 }; // todo @kris-nova get pid, we will probably want to spawn() and wait and remember the pid
                        let status = meta::Status::Complete as i32;
                        let response = ExecutableStatus {
                            meta: Some(meta),
                            proc: Some(proc),
                            status,
                            stdout: String::from_utf8(output.stdout).unwrap(),
                            stderr: String::from_utf8(output.stderr).unwrap(),
                            exit_code: output.status.to_string(),
                        };
                        Ok(Response::new(response))
                    }
                    Err(e) => {
                        let meta = meta::AuraeMeta {
                            name: "-".to_string(),
                            message: format!("{:?}", e),
                        };
                        let proc = meta::ProcessMeta { pid: -1 };
                        let status = meta::Status::Error as i32;
                        let response = ExecutableStatus {
                            meta: Some(meta),
                            proc: Some(proc),
                            status,
                            stdout: "-".to_string(),
                            stderr: "-".to_string(),
                            exit_code: "-".to_string(),
                        };
                        Ok(Response::new(response))
                    }
                }
            }
            Err(e) => {
                let meta = meta::AuraeMeta {
                    name: "-".to_string(),
                    message: format!("{:?}", e),
                };
                let proc = meta::ProcessMeta { pid: -1 };
                let status = meta::Status::Error as i32;
                let response = ExecutableStatus {
                    meta: Some(meta),
                    proc: Some(proc),
                    status,
                    stdout: "-".to_string(),
                    stderr: "-".to_string(),
                    exit_code: "-".to_string(),
                };
                Ok(Response::new(response))
            }
        }
    }

    // async fn executable_stop(
    //     &self,
    //     request: Request<Executable>,
    // ) -> Result<Response<ExecutableStatus>, Status> {
    //     let _r = request.into_inner();
    //     let meta = meta::AuraeMeta {
    //         name: "UNKNOWN_NAME".to_string(),
    //         message: "UNKNOWN_MESSAGE".to_string(),
    //     };
    //     let proc = meta::ProcessMeta { pid: -1 };
    //     let status = meta::Status::Unknown as i32;
    //     let response = ExecutableStatus {
    //         meta: Some(meta),
    //         proc: Some(proc),
    //         status,
    //         stdout: "-".to_string(),
    //         stderr: "-".to_string(),
    //         exit_code: "-".to_string(),
    //     };
    //     Ok(Response::new(response))
    // }
    //
    // async fn container_start(
    //     &self,
    //     _request: Request<Container>,
    // ) -> Result<Response<ContainerStatus>, Status> {
    //     todo!()
    // }
    //
    // async fn container_stop(
    //     &self,
    //     _request: Request<Container>,
    // ) -> Result<Response<ContainerStatus>, Status> {
    //     todo!()
    // }
    //
    // async fn instance_start(
    //     &self,
    //     _request: Request<Instance>,
    // ) -> Result<Response<InstanceStatus>, Status> {
    //     todo!()
    // }
    //
    // async fn instance_stop(
    //     &self,
    //     _request: Request<Instance>,
    // ) -> Result<Response<InstanceStatus>, Status> {
    //     todo!()
    // }
}
