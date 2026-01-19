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

//! A command line tool named "aer" built on the Rust client ("client") that has an
//! identical scope of a single auraed node.
//!
//! This tool is for "power-users" and exists as a way of quickly developing and debugging
//! the APIs as we change them. For example an auraed developer might make a change to
//! an API and need a quick way to test the API locally against a single daemon.

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
#![warn(missing_debug_implementations,
// TODO: missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_extern_crates,
unused_import_braces,
unused_results
)]
#![warn(clippy::unwrap_used)]
// #![warn(missing_docs)] // TODO: We want the docs from the proto

pub mod cri;
pub mod discovery;
pub mod grpc;
pub mod observe;
pub mod runtime;
pub mod vms;

/// Executes an rpc call with the default `Client` and prints the results.
#[macro_export]
macro_rules! execute {
    ($call:path, $req:ident) => {{
        let client = ::client::Client::default().await?;
        let res = $call(&client, $req).await?.into_inner();
        println!("{res:#?}");
        res
    }};
}

/// Executes an rpc call with the default `Client` and prints the results.
/// For use with server streaming requests.
/// The initial response will be printed, followed by printing the stream of messages.
#[macro_export]
macro_rules! execute_server_streaming {
    ($call:path, $req:ident) => {{
        let mut res = $crate::execute!($call, $req);
        while let Some(res) = futures_util::StreamExt::next(&mut res).await {
            let res = res?;
            println!("{res:#?}");
        }
    }};
}
