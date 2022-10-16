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

use anyhow::Result;
//use std::process::Command;

fn main() -> Result<()> {
    // Example running "make command" during the build
    //Command::new("make").args(&["command"]).status().unwrap();

    generate_grpc_code()?;

    Ok(())
}

fn generate_grpc_code() -> Result<()> {
    let mut tonic_builder = tonic_build::configure();

    // Generated services use unwrap. Add them here to suppress the warning.
    for service in ["meta", "observe", "runtime", "schedule"] {
        tonic_builder = tonic_builder
            .server_mod_attribute(service, "#[allow(clippy::unwrap_used)]");
    }

    // Types generated from proto messages derive PartialEq without Eq. Add them here to suppress the warning.
    for message in [
        "meta.AuraeMeta",
        "meta.ProcessMeta",
        "runtime.Executable",
        "runtime.ExecutableStatus",
    ] {
        tonic_builder = tonic_builder.type_attribute(
            message,
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        );
    }

    tonic_builder.compile(
        &[
            "stdlib/v0/meta.proto",
            "stdlib/v0/runtime.proto",
            "stdlib/v0/schedule.proto",
            "stdlib/v0/observe.proto",
        ],
        &["stdlib/v0/"],
    )?;

    Ok(())
}
