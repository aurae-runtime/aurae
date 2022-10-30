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
    bad_style,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    unconditional_recursion,
    unused_comparisons,
    while_true
)]
#![warn(// TODO: missing_copy_implementations,
        // TODO: missing_debug_implementations,
        // TODO: missing_docs,
        trivial_casts,
        trivial_numeric_casts,
        // TODO: unused_extern_crates,
        // TODO: unused_import_braces,
        // TODO: unused_qualifications,
        // TODO: unused_results
        )]
#![warn(clippy::unwrap_used)]

use auraescript::*;
use rhai::{Engine, EvalAltResult, Position};
use std::{env, fs::File, io::Read, path::Path, process::exit};

fn eprint_error(input: &str, mut err: EvalAltResult) {
    #[allow(clippy::unwrap_used)]
    fn eprint_line(lines: &[&str], pos: Position, err_msg: &str) {
        let line = pos.line().expect("position");
        let line_no = format!("{line}: ");

        eprintln!("{}{}", line_no, lines[line - 1]);
        eprintln!(
            "{:>1$} {2}",
            "^",
            line_no.len() + pos.position().unwrap(),
            err_msg
        );
        eprintln!();
    }

    let lines: Vec<_> = input.split('\n').collect();

    // Print error
    let pos = err.take_position();

    if pos.is_none() {
        // No position
        eprintln!("{}", err);
    } else {
        // Specific position
        eprint_line(&lines, pos, &err.to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut contents = String::new();

    for filename in env::args().skip(1) {
        let filename = match Path::new(&filename).canonicalize() {
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

        // Initialize scripting engine
        let mut engine = Engine::new();

        #[cfg(not(feature = "no_optimize"))]
        engine.set_optimization_level(rhai::OptimizationLevel::Simple);

        // Load builtin engine components
        engine = register_stdlib(engine);

        let mut f = match File::open(&filename) {
            Err(err) => {
                eprintln!(
                    "Error reading script file: {}
{}",
                    filename.to_string_lossy(),
                    err
                );
                exit(1);
            }
            Ok(f) => f,
        };

        contents.clear();

        if let Err(err) = f.read_to_string(&mut contents) {
            eprintln!(
                "Error reading script file: {}
{}",
                filename.to_string_lossy(),
                err
            );
            exit(1);
        }

        let contents = if contents.starts_with("#!") {
            // Skip shebang
            &contents[contents.find('\n').unwrap_or(0)..]
        } else {
            &contents[..]
        };

        if let Err(err) = engine
            .compile(contents)
            .map_err(|err| err.into())
            .and_then(|mut ast| {
                ast.set_source(filename.to_string_lossy().to_string());
                engine.run_ast(&ast)
            })
        {
            let filename = filename.to_string_lossy();

            eprintln!("{:=<1$}", "", filename.len());
            eprintln!("{}", filename);
            eprintln!("{:=<1$}", "", filename.len());
            eprintln!();

            eprint_error(contents, *err);
        }
    }
    Ok(())
}
