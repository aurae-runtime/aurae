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

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=\"src\"");

    let out_dir = PathBuf::from("./lib");

    match Command::new("npm").args(["install", "ts-proto"]).status()?.code() {
        Some(0) => {}
        _ => {
            return Err(anyhow!(
                "Command `npm install ts-proto` failed. Is npm installed?"
            ));
        }
    }

    let proto_message_files = ["../api/v0/runtime.proto"];

    for file in proto_message_files {
        let mut out_path = out_dir.clone();
        out_path.push(file.replace(".proto", ".ts"));

        println!("cargo:rerun-if-changed=\"{out_path:?}\"");

        let status = Command::new("protoc")
            .args([
                "--plugin=./node_modules/.bin/protoc-gen-ts_proto",
                &format!("--ts_proto_out={}", out_dir.display()),
                "--ts_proto_opt=outputEncodeMethods=false",
                "--ts_proto_opt=outputClientImpl=false",
                "-I=../api/v0",
                file,
            ])
            .status()?;

        match status.code() {
            Some(0) => {}
            _ => {
                return Err(anyhow!(
                    "Failed to generate Typescript file '{out_path:?}'"
                ))
            }
        }
    }

    generate_grpc_code()?;

    Ok(())
}

fn generate_grpc_code() -> Result<()> {
    let mut tonic_builder = tonic_build::configure();

    // Generated services use unwrap. Add them here to suppress the warning.
    for service in ["observe", "runtime", "schedule"] {
        tonic_builder =
            tonic_builder.server_attribute(service, "#[allow(missing_docs)]");
    }

    // Add proto messages here to add `#[derive(::serde::Serialize, ::serde::Deserialize)]` to the type.
    // If other code generation needs these derive attributes, they will automatically be added.
    let mut derive_serde_serialize_deserialize: Vec<&str> = vec![];

    // Types generated from proto messages derive PartialEq without Eq. Add them here to suppress the warning.
    for message in [
        "runtime.Cell",
        "runtime.Executable",
        "runtime.ExecutableReference",
        "runtime.AllocateCellRequest",
        "runtime.AllocateCellResponse",
        "runtime.FreeCellRequest",
        "runtime.FreeCellResponse",
        "runtime.StartCellRequest",
        "runtime.StartCellResponse",
        "runtime.StopCellRequest",
        "runtime.StopCellResponse",
    ] {
        tonic_builder = tonic_builder.type_attribute(
            message,
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        );

        derive_serde_serialize_deserialize.push(message);
    }

    // Loop to add serde attributes
    for message in derive_serde_serialize_deserialize {
        tonic_builder = tonic_builder
            .type_attribute(
                message,
                "#[derive(::serde::Serialize, ::serde::Deserialize)]",
            )
            .type_attribute(message, "#[serde(default)]");
    }

    tonic_builder
        .build_server(false)
        .compile(&["../api/v0/runtime.proto"], &["../api/v0"])?;

    Ok(())
}
