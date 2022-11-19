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
        // TODO: unused_results
        )]
// The project prefers .expect("reason") instead of .unwrap() so we fail
// on any .unwrap() statements in the code.
#![warn(clippy::unwrap_used)]
#![allow(clippy::extra_unused_lifetimes)]
//#![warn(missing_docs)]

pub mod builtin;
pub mod runtime;
use crate::builtin::client::*;
use deno_core::*;

// --[ Main Standard Library Functions ]--

#[op]
fn ae_connect() -> Result<AuraeClient, error::AnyError> {
    let client = connect();
    Ok(client)
}

#[op]
fn ae_runtime_cell_service_allocate() -> Result<String, error::AnyError> {
    Ok("".parse()?)
}

#[op]
fn ae_runtime_cell_service_start() -> Result<String, error::AnyError> {
    Ok("".parse()?)
}

#[op]
fn ae_runtime_cell_service_stop() -> Result<String, error::AnyError> {
    Ok("".parse()?)
}

#[op]
fn ae_runtime_cell_service_free() -> Result<String, error::AnyError> {
    Ok("".parse()?)
}

// --[ Preloader ]--

// Similar to LD_PRELOAD or kprobe in the kernel. This function
// is executed for *every* op function.
//
// Can be used to intercept functions to create magic "behind the scenes".
fn middleware_intercept(decl: OpDecl) -> OpDecl {
    //println!("{:?} {:?}", decl.name, decl.argc);
    decl
}

// The main registry for AuraeScript
//
// All new functionality must go through here.
pub fn register_stdlib() -> Extension {
    let ext = Extension::builder()
        // Standard Operations
        .ops(vec![
            ae_connect::decl(),
            ae_runtime_cell_service_allocate::decl(),
            ae_runtime_cell_service_start::decl(),
            ae_runtime_cell_service_stop::decl(),
            ae_runtime_cell_service_free::decl(),
        ])
        .middleware(middleware_intercept)
        .build();
    ext
}

// --[ Tests ]--

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake() {
        //assert_eq!(1, 2); // Flake test check
    }
}
