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
// pub mod runtime;
use rhai::Engine;

use crate::builtin::client::*;
use crate::builtin::*;
use crate::observe::*;

pub fn register_stdlib(mut engine: Engine) -> Engine {
    engine
        //
        // Top Level Commands
        .register_fn("about", about)
        .register_fn("connect", connect)
        //
        // Client
        .register_type_with_name::<AuraeClient>("AuraeClient")
        .register_fn("new", AuraeClient::new)
        .register_fn("info", AuraeClient::info)
        //
        // Runtime
        //.register_fn("runtime", AuraeClient::runtime)
        //
        // Observe
        .register_type_with_name::<Observe>("Observe")
        .register_fn("observe", AuraeClient::observe)
        .register_fn("new", Observe::new)
        .register_fn("status", Observe::status)
        //
        // Version
        .register_fn("version", version);

    return engine;
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
