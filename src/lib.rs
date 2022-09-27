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

// Issue tracking: https://github.com/rust-lang/rust/issues/85410
// Here we need to build an abstract socket from a SocketAddr until
// tokio supports abstract sockets natively

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
        .register_fn("runtime", AuraeClient::runtime)
        // Runtime->Executable
        .register_type_with_name::<Executable>("Executable")
        .register_fn("exec", exec)
        .register_fn("json", Executable::json)
        .register_fn("raw", Executable::raw)
        .register_get_set("name", Executable::get_name, Executable::set_name)
        .register_get_set("exec", Executable::get_exec, Executable::set_exec)
        .register_get_set(
            "comment",
            Executable::get_comment,
            Executable::set_comment,
        )
        .register_type_with_name::<StartExecutableResponse>(
            "StartExecutableResponse",
        )
        .register_fn("json", StartExecutableResponse::json)
        .register_fn("raw", StartExecutableResponse::raw)
        .register_fn("start_executable", Runtime::start_executable)
        .register_fn("start", Runtime::start_executable) // alias
        .register_type_with_name::<StopExecutableResponse>(
            "StopExecutableResponse",
        )
        .register_fn("json", StopExecutableResponse::json)
        .register_fn("raw", StopExecutableResponse::raw)
        .register_fn("stop_executable", Runtime::stop_executable)
        .register_fn("stop", Runtime::stop_executable) // alias
        .register_type_with_name::<RegisterExecutableResponse>(
            "RegisterExecutableResponse",
        )
        .register_fn("json", RegisterExecutableResponse::json)
        .register_fn("raw", RegisterExecutableResponse::raw)
        .register_fn("register_executable", Runtime::register_executable)
        .register_fn("register", Runtime::register_executable) // alias
        .register_type_with_name::<DestroyExecutableResponse>(
            "StartExecutableResponse",
        )
        .register_fn("json", DestroyExecutableResponse::json)
        .register_fn("raw", DestroyExecutableResponse::raw)
        .register_fn("destroy_executable", Runtime::destroy_executable)
        .register_fn("destroy", Runtime::destroy_executable) //alias
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
