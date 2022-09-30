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

//use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example running "make command" during the build
    //Command::new("make").args(&["command"]).status().unwrap();

    // gRPC
    tonic_build::configure()
        .type_attribute(
            "meta.AuraeMeta",
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        )
        .type_attribute(
            "runtime.Executable",
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        )
        .type_attribute(
            "runtime.ExecutableStatus",
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        )
        .type_attribute(
            "meta.AuraeMeta",
            "#[derive(::serde::Serialize, ::serde::Deserialize, ::macros::Output)]",
        )
        .type_attribute(
            "observe.StatusRequest",
            "#[derive(::serde::Serialize, ::serde::Deserialize, ::macros::Output)]",
        )
        .type_attribute(
            "observe.StatusResponse",
            "#[derive(::serde::Serialize, ::serde::Deserialize, ::macros::Output)]",
        )
        .type_attribute(
            "runtime.Executable",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "runtime.ExecutableStatus",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(
            &[
                "../api/v1/meta.proto",
                "../api/v1/runtime.proto",
                "../api/v1/observe.proto",
            ],
            &["../api/v1"],
        )?;
    Ok(())
}
