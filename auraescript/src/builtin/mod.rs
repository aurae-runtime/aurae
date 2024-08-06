/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */

//! The builtin functionality for AuraeScript.
//!
//! AuraeScript has a small amount of magic with regard to authentication and
//! managing the client and requests, responses, and output.
//!
//! Most of the built-in logic that makes AuraeScript useful to an end-user
//! lives in this module.

pub(crate) mod auraescript_client;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Show meta information about AuraeScript.
#[allow(unused)]
fn about() {
    println!("\n");
    println!("Aurae. Distributed Runtime.");
    println!("Authors: {AUTHORS}");
    version();
    println!("\n");
}

/// Show version information.
fn version() {
    println!("Version: {VERSION}");
}