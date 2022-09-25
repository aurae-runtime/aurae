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

pub mod client;
pub mod codes;
pub mod config;

//mod x509_certificate;
//use x509_certificate::X509Certificate;

// Cargo passes environmental variables to the compiler.
// We can access meta data from Cargo.toml directly here.
//
// Here are the values:
//
// CARGO_MANIFEST_DIR
// CARGO_PKG_AUTHORS
// CARGO_PKG_DESCRIPTION
// CARGO_PKG_HOMEPAGE
// CARGO_PKG_NAME
// CARGO_PKG_REPOSITORY
// CARGO_PKG_VERSION
// CARGO_PKG_VERSION_MAJOR
// CARGO_PKG_VERSION_MINOR
// CARGO_PKG_VERSION_PATCH
// CARGO_PKG_VERSION_PRE
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub fn about() {
    println!("\n");
    println!("Aurae. Distributed Runtime.");
    println!("Authors: {}", AUTHORS);
    version();
    println!("\n");
}

pub fn version() {
    println!("Version: {}", VERSION);
}
