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

#![allow(dead_code)]

use aurae_proto::runtime::{
    cell_service_server, AllocateCellRequest, AllocateCellResponse,
    FreeCellRequest, FreeCellResponse, StartCellRequest, StartCellResponse,
    StopCellRequest, StopCellResponse,
};
use tonic::{Request, Response, Status};
use validation::ValidatingType;

mod allocate;

tonic::include_proto!("runtime");

#[derive(Debug, Default, Clone)]
pub struct CellService {}

#[tonic::async_trait]
impl cell_service_server::CellService for CellService {
    async fn allocate(
        &self,
        request: Request<AllocateCellRequest>,
    ) -> std::result::Result<Response<AllocateCellResponse>, Status> {
        // TODO: Try to generate boilerplate code that:
        //  - extracts context (`request.extensions()`) from the request; we don't set this yet
        //  - validates the request message
        //  - calls `request.execute(&context)`
        //      - references are used to allow for retrying the request on transient errors
        //      - if we don't want to handle retries, we should pass ownership for efficiency
        //  - ideally has variations for all rpc types (req/res, client/server/bidirectional streaming)
        //
        // *Feel free to delete this task. The intention of including it is to share my (future-highway) thoughts (spreading margarine 😂) on how to evolve the codebase to make "doing the right thing" easier.

        let request = request.into_inner().validate(None).map_err(|e| {
            // TODO: impl From<ValidationError> for Status
            //  - Google suggests usage of their rich error model (https://grpc.io/docs/guides/error/)
            Status::invalid_argument(format!("Validation failure: {e}"))
        })?;

        request.execute().await
    }

    async fn free(
        &self,
        _request: Request<FreeCellRequest>,
    ) -> std::result::Result<Response<FreeCellResponse>, Status> {
        todo!()
    }

    async fn start(
        &self,
        _request: Request<StartCellRequest>,
    ) -> std::result::Result<Response<StartCellResponse>, Status> {
        todo!()
    }

    async fn stop(
        &self,
        _request: Request<StopCellRequest>,
    ) -> std::result::Result<Response<StopCellResponse>, Status> {
        todo!()
    }
}

// async fn run_executable(
//     &self,
//     request: Request<Executable>,
// ) -> Result<Response<ExecutableStatus>, Status> {
//     let r = request.into_inner();
//     let cmd = command_from_string(&r.command);
//     match cmd {
//         Ok(mut cmd) => {
//             let output = cmd.output();
//             match output {
//                 Ok(output) => {
//                     let meta = meta::AuraeMeta {
//                         name: r.command,
//                         message: "-".to_string(),
//                     };
//                     let proc = meta::ProcessMeta { pid: -1 }; // todo @kris-nova get pid, we will probably want to spawn() and wait and remember the pid
//                     let status = meta::Status::Complete as i32;
//                     let response = ExecutableStatus {
//                         meta: Some(meta),
//                         proc: Some(proc),
//                         status,
//                         stdout: String::from_utf8(output.stdout)
//                             .expect("reading stdout"),
//                         stderr: String::from_utf8(output.stderr)
//                             .expect("reading stderr"),
//                         exit_code: output.status.to_string(),
//                     };
//                     Ok(Response::new(response))
//                 }
//                 Err(e) => {
//                     let meta = meta::AuraeMeta {
//                         name: "-".to_string(),
//                         message: format!("{:?}", e),
//                     };
//                     let proc = meta::ProcessMeta { pid: -1 };
//                     let status = meta::Status::Error as i32;
//                     let response = ExecutableStatus {
//                         meta: Some(meta),
//                         proc: Some(proc),
//                         status,
//                         stdout: "-".to_string(),
//                         stderr: "-".to_string(),
//                         exit_code: "-".to_string(),
//                     };
//                     Ok(Response::new(response))
//                 }
//             }
//         }
//         Err(e) => {
//             let meta = meta::AuraeMeta {
//                 name: "-".to_string(),
//                 message: format!("{:?}", e),
//             };
//             let proc = meta::ProcessMeta { pid: -1 };
//             let status = meta::Status::Error as i32;
//             let response = ExecutableStatus {
//                 meta: Some(meta),
//                 proc: Some(proc),
//                 status,
//                 stdout: "-".to_string(),
//                 stderr: "-".to_string(),
//                 exit_code: "-".to_string(),
//             };
//             Ok(Response::new(response))
//         }
//     }
// }

// async fn run_cell(
//     &self,
//     _request: Request<Cell>,
// ) -> Result<Response<CellStatus>, Status> {
//     todo!();
//     // let syscall = create_syscall();
//     // let mut container =
//     //     ContainerBuilder::new("123".to_string(), syscall.as_ref())
//     //         .as_init(PathBuf::new())
//     //         .with_systemd(false)
//     //         .build()
//     //         .expect("building container");
//     // // .with_pid_file(args.pid_file.as_ref())?
//     // // .with_console_socket(args.console_socket.as_ref())
//     // // .with_root_path(root_path)?
//     // // .with_preserved_fds(args.preserve_fds)
//     // // .as_init(&args.bundle)
//     // // .with_systemd(false)
//     // // .build()?;
//     //
//     // let _ = container.start();
//     // let meta =
//     //     meta::AuraeMeta { name: "-".to_string(), message: "-".to_string() };
//     // let status = meta::Status::Complete as i32;
//     // let container_statuses = vec![ContainerStatus {
//     //     meta: Some(meta::AuraeMeta {
//     //         name: "-".to_string(),
//     //         message: "-".to_string(),
//     //     }),
//     //     status: meta::Status::Complete as i32,
//     //     proc: Some(meta::ProcessMeta { pid: -1 }),
//     // }];
//     // let response =
//     //     CellStatus { meta: Some(meta), status, container_statuses };
//     // Ok(Response::new(response))
// }

// async fn function_name(
//     &self,
//     _request: Request<Container>,
// ) -> Result<Response<ContainerStatus>, Status> {
//     todo!()
// }
