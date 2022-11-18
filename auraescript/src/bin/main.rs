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

// TODO @kris-nova as we move to Deno we probably want to revist the main function
// Lint groups: https://doc.rust-lang.org/rustc/lints/groups.html
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    unconditional_recursion,
    unused_comparisons,
    while_true
)]
#![warn(// TODO: missing_debug_implementations,
// TODO: missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_qualifications,
// TODO: unused_results
)]
#![warn(clippy::unwrap_used)]

use auraescript::register_stdlib;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use std::io::Read;
use std::{env, fs::File, path::Path, process::exit};

fn main() -> anyhow::Result<()> {
    // Import the "Standard Library" from lib.rs
    let ext = register_stdlib();

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        ..Default::default()
    });

    // Load the scripts from the arguments
    let mut contents = String::new();

    for filename in env::args().skip(1) {
        match Path::new(&filename).canonicalize() {
            Err(err) => {
                eprintln!(
                    "Error script file path: {}
{}",
                    filename, err
                );
                exit(1);
            }
            Ok(f) => {
                match f.strip_prefix(std::env::current_dir()?.canonicalize()?) {
                    Ok(f) => f.into(),
                    _ => f,
                }
            }
        };

        // Open each file
        let mut f = match File::open(&filename) {
            Err(err) => {
                eprintln!(
                    "Error reading script file: {}
{}",
                    filename, err
                );
                exit(1);
            }
            Ok(f) => f,
        };

        // Clear the contents of the script, and read the new contents
        contents.clear();
        if let Err(err) = f.read_to_string(&mut contents) {
            eprintln!(
                "Error reading script file: {}
{}",
                filename, err
            );
            exit(1);
        }

        let contents = if contents.starts_with("#!") {
            // Skip shebang
            &contents[contents.find('\n').unwrap_or(0)..]
        } else {
            &contents[..]
        };

        // Add the standard library to each "script" that is being executed.
        let stdlib = include_str!("../../lib/aurae.ae");
        let mut script_to_exec: String = stdlib.to_owned();
        script_to_exec.push_str(contents);

        runtime.execute_script(&filename, &script_to_exec).expect("runtime");
    }
    Ok(())
}
