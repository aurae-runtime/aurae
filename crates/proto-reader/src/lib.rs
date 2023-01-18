/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

pub mod helpers;

use protobuf_parse::{ParsedAndTypechecked, Parser};
use std::path::PathBuf;
use syn::Lit;

/// Returns a tuple of a [PathBuf] to the proto file and the [ParsedAndTypechecked] output.
pub fn parse(file_path: Lit) -> (PathBuf, ParsedAndTypechecked) {
    let crate_root = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        _ => panic!("env variable 'CARGO_MANIFEST_DIR' was not set. Failed to find crate root"),
    };

    let parsed_file_path = match &file_path {
        Lit::Str(file_path) => {
            let file_path = file_path.value();
            let file_path = file_path.trim_matches('"');

            let file_path = crate_root.join(file_path);

            file_path.canonicalize().unwrap_or_else(|e| {
                panic!(
                    "failed to determine absolute path for {file_path:?}: {e}"
                )
            })
        }
        _ => panic!("expected literal string with path to proto file in the api directory"),
    };

    let mut api_dir = parsed_file_path.clone();
    let api_dir = loop {
        match api_dir.parent() {
            Some(parent) => {
                if parent.is_dir() && parent.ends_with("api") {
                    break parent;
                } else {
                    api_dir = parent.to_path_buf();
                }
            }
            _ => panic!("proto file not in api directory"),
        }
    };

    let proto = Parser::new()
        .protoc()
        .protoc_extra_args(["--experimental_allow_proto3_optional"])
        .include(api_dir)
        .input(&parsed_file_path)
        .parse_and_typecheck()
        .expect("failed to parse proto file");

    (parsed_file_path, proto)
}
