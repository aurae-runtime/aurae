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
#![warn(
    missing_debug_implementations,
    // TODO: missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results
)]
#![warn(clippy::unwrap_used)]

use auraed::{prep_oci_spec_for_spawn, run, AuraedRuntime};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};

/// Default exit code for successful termination of auraed.
pub const EXIT_OKAY: i32 = 0;

/// Default exit code for a runtime error of auraed.
pub const EXIT_ERROR: i32 = 1;

/// Command line options for auraed.
///
/// Defines the configurable options which can be used to populate
/// an AuraeRuntime structure.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct AuraedOptions {
    /// The signed server certificate. Defaults to /etc/aurae/pki/_signed.server.crt
    #[clap(long, value_parser)]
    server_crt: Option<String>,
    /// The secret server key. Defaults to /etc/aurae/pki/server.key
    #[clap(long, value_parser)]
    server_key: Option<String>,
    /// The CA certificate. Defaults to /etc/aurae/pki/ca.crt
    #[clap(long, value_parser)]
    ca_crt: Option<String>,
    /// Aurae socket address.  Depending on context, this should be a file or a network address.
    /// Defaults to ${runtime_dir}/aurae.sock or [::1]:8080 respectively.
    ///
    /// Warning: This socket is created (by default) with user
    /// mode 0o766 which allows for unprivileged access to the
    /// auraed daemon which can in turn be used to execute privileged
    /// processes and commands. Access to the socket must be governed
    /// by an appropriate mTLS Authorization setting in order to maintain
    /// a secure multi tenant system.
    #[clap(short, long, value_parser)]
    socket: Option<String>,
    /// Aurae runtime path.  Defaults to /var/run/aurae.
    ///
    /// Here is where the auraed daemon will store artifacts such as
    /// OCI bundles for containers, the aurae.sock socket file, and
    /// runtime pod configuration.
    ///
    /// This is the main "runtime" location for all artifacts that are
    /// a consequence of runtime operations.
    ///
    /// All aspects of the auraed daemon should respect this value.
    #[clap(short, long, value_parser)]
    runtime_dir: Option<String>,
    /// Aurae library path. Defaults to /var/lib/aurae
    ///
    /// Here is where the daemon will look for artifacts such as eBPF
    /// bytecode (ELF objects/probes) and other dependencies that can
    /// optionally be included at runtime.
    ///
    /// All aspects of the auraed library and dependency artifacts
    /// should respect this value.
    #[clap(short, long, value_parser)]
    library_dir: Option<String>,
    /// Toggle verbosity. Default false
    #[clap(short, long, alias = "ritz")]
    verbose: bool,
    /// Run auraed as a nested instance of itself in an Aurae cell.
    #[clap(long)]
    nested: bool,
    // Subcommands for the project
    #[clap(subcommand)]
    subcmd: Option<SubCommands>,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    Spawn {
        #[clap(short, long, value_parser, default_value = ".")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments into AuraedOptions
    let options = AuraedOptions::parse();

    // Match on the subcommand and handle accordingly
    let exit_code = match &options.subcmd {
        Some(SubCommands::Spawn { output }) => {
            handle_spawn_subcommand(output).await
        }
        None => handle_default(options).await,
    };

    std::process::exit(exit_code);
}

async fn handle_default(options: AuraedOptions) -> i32 {
    info!("Starting Aurae Daemon Runtime");
    info!("Aurae Daemon is pid {}", std::process::id());

    // Destructure the options into individual variables
    let AuraedOptions {
        server_crt,
        server_key,
        ca_crt,
        socket,
        runtime_dir,
        library_dir,
        verbose,
        nested,
        subcmd: _,
    } = options;

    // Destructure the default runtime into individual variables
    let AuraedRuntime {
        auraed: default_auraed,
        ca_crt: default_ca_crt,
        server_crt: default_server_crt,
        server_key: default_server_key,
        runtime_dir: default_runtime_dir,
        library_dir: default_library_dir,
    } = AuraedRuntime::default();

    // Create a new runtime configuration, using provided options or defaults
    let runtime = AuraedRuntime {
        auraed: default_auraed,
        ca_crt: ca_crt.map(PathBuf::from).unwrap_or(default_ca_crt),
        server_crt: server_crt.map(PathBuf::from).unwrap_or(default_server_crt),
        server_key: server_key.map(PathBuf::from).unwrap_or(default_server_key),
        runtime_dir: runtime_dir
            .map(PathBuf::from)
            .unwrap_or(default_runtime_dir),
        library_dir: library_dir
            .map(PathBuf::from)
            .unwrap_or(default_library_dir),
    };

    // Run the auraed daemon with the configured runtime
    if let Err(e) = run(runtime, socket, verbose, nested).await { 
        error!("{:?}", e); // Log any errors that occur
        EXIT_ERROR // Return error exit code
    } else {
        EXIT_OKAY // Return success exit code
    }
}

async fn handle_spawn_subcommand(output: &str) -> i32 {
    info!("Spawning Auraed OCI bundle: {}", output);
    prep_oci_spec_for_spawn(output); // Prepare the OCI spec for spawning
    EXIT_OKAY // Return success exit code
}