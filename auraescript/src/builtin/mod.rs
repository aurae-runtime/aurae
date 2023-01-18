/* -------------------------------------------------------------------------- *\
#             Apache 2.0 License Copyright © The Aurae Authors                #
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

//! The builtin functionality for AuraeScript.
//!
//! AuraeScript has a small amount of magic with regard to authentication and
//! managing the client and requests, responses, and output.
//!
//! Most of the built-in logic that makes AuraeScript useful to an end-user
//! lives in this module.

#[allow(dead_code)]
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[allow(dead_code)]
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Show meta information about AuraeScript.
#[allow(dead_code)]
fn about() {
    println!("\n");
    println!("Aurae. Distributed Runtime.");
    println!("Authors: {}", AUTHORS);
    version();
    println!("\n");
}

/// Show version information.
#[allow(dead_code)]
fn version() {
    println!("Version: {}", VERSION);
}
