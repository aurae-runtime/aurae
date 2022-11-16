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
        tonic_builder =
            tonic_builder.server_attribute(service, "#[allow(missing_docs)]");
    }

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
        "observe.GetAuraeDaemonLogStreamRequest",
        "observe.GetSubProcessStreamRequest",
        "observe.LogItem",
    ] {
        tonic_builder = tonic_builder.type_attribute(
            message,
            "#[allow(clippy::derive_partial_eq_without_eq)]",
        );
    }

    // Add proto messages here to add `#[derive(::serde::Serialize, ::serde::Deserialize)]` to the type.
    // If other code generation (e.g., `::macros::Output`) needs these derive attributes, they will automatically be added.
    let mut derive_serde_serialize_deserialize: Vec<&str> = vec![];

    // Add proto messages here to generate output functions (e.g., `json()`) on the type
    for message in ["runtime.Cell", "runtime.Executable", "observe.LogItem"] {
        if !derive_serde_serialize_deserialize.contains(&message) {
            derive_serde_serialize_deserialize.push(message);
        }

        tonic_builder = tonic_builder
            .type_attribute(message, "#[derive(::macros::Output)]");
    }

    // Add proto messages  here to generate getters and setters.
    tonic_builder = generate_grpc_code_for_getters_setters(
        tonic_builder,
        vec![GetSet {
            message: "runtime.Executable",
            ignore_fields: vec!["meta"],
            ..Default::default()
        }],
    );

    // Loop to add serde attributes
    for message in derive_serde_serialize_deserialize {
        tonic_builder = tonic_builder.type_attribute(
            message,
            "#[derive(::serde::Serialize, ::serde::Deserialize)]",
        )
    }

    tonic_builder.build_server(false).compile(
        &[
            "../api/v0/runtime.proto",
            "../api/v0/observe.proto",
            "../api/v0/schedule.proto",
        ],
        &["../api/v0"],
    )?;

    Ok(())
}

#[allow(clippy::single_element_loop)]
fn generate_grpc_code_for_getters_setters(
    mut tonic_builder: tonic_build::Builder,
    message_settings: Vec<GetSet>,
) -> tonic_build::Builder {
    // Add proto messages here to generate getters and setters.
    for getset in message_settings {
        let attribute = match (getset.getters, getset.setters) {
            (true, true) => "#[derive(::macros::Getters, ::macros::Setters)]",
            (true, false) => "#[derive(::macros::Getters)]",
            (false, true) => "#[derive(::macros::Setters)]",
            (false, false) => continue,
        };

        tonic_builder = tonic_builder.type_attribute(getset.message, attribute);

        for field in getset.ignore_fields {
            let path = format!("{}.{}", getset.message, field);
            tonic_builder =
                tonic_builder.field_attribute(path, "#[getset(ignore)]");
        }

        for field in getset.getters_ignore_fields {
            let path = format!("{}.{}", getset.message, field);
            tonic_builder =
                tonic_builder.field_attribute(path, "#[getset(ignore_get)]");
        }

        for field in getset.setters_ignore_fields {
            let path = format!("{}.{}", getset.message, field);
            tonic_builder =
                tonic_builder.field_attribute(path, "#[getset(ignore_set)]");
        }
    }

    tonic_builder
}

struct GetSet {
    /// The name of the proto message
    message: &'static str,
    /// Field names to ignore (no get or set function)
    ignore_fields: Vec<&'static str>,
    /// Should getters be generated
    getters: bool,
    /// Field names to ignore only when generating getters
    getters_ignore_fields: Vec<&'static str>,
    /// Should setters be generated
    setters: bool,
    /// Field names to ignore only when generating setters
    setters_ignore_fields: Vec<&'static str>,
}

impl Default for GetSet {
    fn default() -> Self {
        Self {
            message: "",
            ignore_fields: vec![],
            getters: true,
            getters_ignore_fields: vec![],
            setters: true,
            setters_ignore_fields: vec![],
        }
    }
}
