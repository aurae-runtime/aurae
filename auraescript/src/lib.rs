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

//! An interpreted infrastructure language built for enterprise platform teams.
//!
//! AuraeScript is an opinionated and Turing complete client language for an
//! Aurae server. AuraeScript is an alternative to templated YAML for teams
//! to express their applications.
//!
//! The AuraeScript definition lives in this crate library (lib.rs).
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
#![warn(// TODO: missing_copy_implementations,
        // TODO: missing_debug_implementations,
        // TODO: missing_docs,
        trivial_casts,
        trivial_numeric_casts,
        // TODO: unused_extern_crates,
        unused_import_braces,
        // TODO: unused_qualifications,
        // TODO: unused_results
        )]
// The project prefers .expect("reason") instead of .unwrap() so we fail
// on any .unwrap() statements in the code.
#![warn(clippy::unwrap_used)]

// AuraeScript has a high expectation for documentation in this library as it
// is used for the scripting language docs directly.
#[warn(missing_docs)]
pub mod builtin;
pub mod meta;
pub mod observe;
pub mod runtime;
pub mod schedule;

use rhai::Engine;

use crate::builtin::client::*;
use crate::builtin::*;
use crate::observe::*;
use crate::runtime::*;
use crate::schedule::*;

/// AuraeScript Standard Library.
///
/// The main definition of functions, objects, types, methods, and values
/// for the entire AuraeScript library.
///
/// A large portion of this library is plumbed through from the gRPC client.
///
/// An important note for this library is that it is not "1-to-1" with the
/// gRPC client.
///
/// There are carefully chosen subtle and semantic differences in how various
/// parts of the Aurae Standard Library are exposed with AuraeScript.
/// The philosophy is to keep the library beautiful, and simple.
/// We prefer name() or meaningful_verbose_name()
/// We prefer exec() over executable()
///
/// Each function in here must be heavily scrutinized as we will need to
/// maintain some semblance of backward compatability over time.
pub fn register_stdlib(mut engine: Engine) -> Engine {
    engine
        // about function
        //
        // Reserved function name to share information about the current
        // client interpreter.
        .register_fn("about", about)
        // connect function
        //
        // Opinionated function that will attempt to look up configuration
        // according to the semantics defined in the configuration module.
        // This function will load the system configuration that is available
        // in well known locations, or it will fail with a non-JSON error!
        .register_fn("connect", connect)
        // AuraeClient type
        //
        // Returned from connect() and is the pointer to the client which
        // can be used to initialize subsystems with Aurae.
        .register_type_with_name::<AuraeClient>("AuraeClient")
        // AuraeClient.info function
        //
        // Used to show information about a specific client.
        .register_fn("info", AuraeClient::info)
        // X509Details type
        //
        // Identity and mTLS details.
        .register_type_with_name::<X509Details>("X509Details")
        .register_fn("json", X509Details::json)
        .register_fn("raw", X509Details::raw)
        // Runtime type
        //
        // The runtime subsystem with corresponding methods.
        .register_type_with_name::<Core>("Core")
        .register_fn("runtime", AuraeClient::runtime)
        // Executable type
        //
        // An executable which can be started, stopped, or scheduled.
        .register_type_with_name::<Executable>("Executable")
        // exec function
        //
        // Most efficient way to execute a command with Aurae. Wraps up
        // the runtime subsystem, and cmd setting into a single alias.
        // Execute the argument string synchronously without any other
        // code required.
        .register_fn("exec", exec)
        // cmd function
        //
        // Create an instance of an Executable type with an argument
        // command string. Can be passed to various subsystems.
        .register_fn("cmd", cmd)
        // run_executable function
        //
        // Direct access to the run_executable function.
        .register_fn("run_executable", Core::run_executable)
        .register_fn("json", Executable::json)
        .register_fn("raw", Executable::raw)
        .register_get_set(
            "command",
            Executable::get_command,
            Executable::set_command,
        )
        .register_get_set(
            "comment",
            Executable::get_comment,
            Executable::set_comment,
        )
        // ExecutableStatus type
        //
        // Response with the status of a given Executable back from an Aurae server.
        .register_type_with_name::<ExecutableStatus>("ExecutableStatus")
        .register_fn("json", ExecutableStatus::json)
        .register_fn("raw", ExecutableStatus::raw)
        // ScheduleExecutable type
        //
        // The ScheduleExecutable subsystem.
        .register_type_with_name::<ScheduleExecutable>("ScheduleExecutable")
        .register_fn("schedule_executable", AuraeClient::schedule_executable)
        // ScheduleExecutable.enable function
        //
        // Enable an Executable type{} to always run on the system.
        .register_fn("enable", ScheduleExecutable::enable)
        // ScheduleExecutable.disable function
        //
        // Disable an Executable type{} to not run on the system.
        .register_fn("disable", ScheduleExecutable::disable)
        // ScheduleExecutable.destroy function
        //
        // Destroy an Executable type{} from the system record.
        .register_fn("destroy", ScheduleExecutable::destroy)
        .register_type_with_name::<ExecutableEnableResponse>(
            "ExecutableEnableResponse",
        )
        .register_fn("json", ExecutableEnableResponse::json)
        .register_fn("raw", ExecutableEnableResponse::raw)
        .register_type_with_name::<ExecutableDisableResponse>(
            "ExecutableDisableResponse",
        )
        .register_fn("json", ExecutableDisableResponse::json)
        .register_fn("raw", ExecutableDisableResponse::raw)
        .register_type_with_name::<ExecutableDestroyResponse>(
            "ExecutableDestroyResponse",
        )
        .register_fn("json", ExecutableDestroyResponse::json)
        .register_fn("raw", ExecutableDestroyResponse::raw)
        // Observe type
        //
        // The observe subsystem.
        .register_type_with_name::<Observe>("Observe")
        .register_fn("observe", AuraeClient::observe)
        // Observe.status function
        //
        // Retrieve total system status metrics.
        .register_fn("status", Observe::status_default)
        .register_type_with_name::<StatusResponse>("StatusResponse")
        .register_fn("json", StatusResponse::json)
        .register_fn("raw", StatusResponse::raw)
        // version function
        //
        // The Aurae version running on
        .register_fn("version", version);

    engine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine() {
        let mut engine = Engine::new();
        engine = register_stdlib(engine);
        let sigs = engine.gen_fn_signatures(true);
        println!("{:?}", sigs);
    }

    #[test]
    fn test_flake() {
        //assert_eq!(1, 2); // Flake test check
    }
}
