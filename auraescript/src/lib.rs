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

use anyhow::anyhow;
use deno_core::Extension;

// This is a hack to make the `#[op]` macro work with
// deno_core examples.
// You can remove this:
use deno_core::*;

#[op]
fn op_example_print() -> Result<(), deno_core::error::AnyError> {
    println!("Example printing...");
    Ok(())
}

#[op]
fn op_example_error() -> Result<(), deno_core::error::AnyError> {
    Err(anyhow!("Example error: {}", "value"))
}

pub fn register_stdlib() -> Extension {
    let ext = Extension::builder()
        .ops(vec![op_example_print::decl(), op_example_error::decl()])
        .build();
    ext
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake() {
        //assert_eq!(1, 2); // Flake test check
    }
}
