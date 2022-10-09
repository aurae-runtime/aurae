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

#![warn(clippy::unwrap_used)]

pub mod builtin;
pub mod meta;
pub mod observe;
pub mod runtime;

use rhai::Engine;

use crate::builtin::client::*;
use crate::builtin::*;
use crate::observe::*;
use crate::runtime::*;

pub fn register_stdlib(mut engine: Engine) -> Engine {
    engine
        //
        // Top Level Commands
        .register_fn("about", about)
        .register_fn("connect", connect)
        //
        //
        // Client
        .register_type_with_name::<AuraeClient>("AuraeClient")
        .register_fn("info", AuraeClient::info)
        .register_type_with_name::<X509Details>("X509Details")
        .register_fn("json", X509Details::json)
        .register_fn("raw", X509Details::raw)
        //
        // Runtime
        .register_type_with_name::<Runtime>("Runtime")
        //
        // Executable
        .register_fn("runtime", AuraeClient::runtime)
        .register_type_with_name::<Executable>("Executable")
        .register_fn("exec", exec)
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
        //
        // ExecutableStatus
        .register_type_with_name::<ExecutableStatus>("ExecutableStatus")
        .register_fn("json", ExecutableStatus::json)
        .register_fn("raw", ExecutableStatus::raw)
        //
        // Start Executable
        .register_fn("executable_Start", Runtime::executable_start)
        .register_fn("start", Runtime::executable_start) // alias
        //
        // Stop Executable
        .register_fn("stop_executable", Runtime::executable_stop)
        .register_fn("stop", Runtime::executable_stop) // alias
        //
        // Observe
        .register_type_with_name::<Observe>("Observe")
        .register_fn("observe", AuraeClient::observe)
        .register_fn("status", Observe::status)
        .register_type_with_name::<StatusResponse>("StatusResponse")
        .register_fn("json", StatusResponse::json)
        .register_fn("raw", StatusResponse::raw)
        //
        // Version
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
