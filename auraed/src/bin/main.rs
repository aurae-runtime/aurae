/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
    unused_qualifications,
    unused_results
)]

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
    let options = AuraedOptions::parse();

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

    let mut runtime = AuraedRuntime::default();

    if let Some(ca_cert) = options.ca_crt {
        runtime.ca_crt = PathBuf::from(ca_cert);
    }

    if let Some(server_crt) = options.server_crt {
        runtime.ca_crt = PathBuf::from(server_crt);
    }

    if let Some(server_key) = options.server_key {
        runtime.ca_crt = PathBuf::from(server_key);
    }

    if let Some(runtime_dir) = options.runtime_dir {
        runtime.ca_crt = PathBuf::from(runtime_dir);
    }

    if let Some(library_dir) = options.library_dir {
        runtime.ca_crt = PathBuf::from(library_dir);
    }

    if let Err(e) =
        run(runtime, options.socket, options.verbose, options.nested).await
    {
        error!("{:?}", e);
        EXIT_ERROR
    } else {
        EXIT_OKAY
    }
}

async fn handle_spawn_subcommand(output: &str) -> i32 {
    info!("Spawning Auraed OCI bundle: {}", output);
    prep_oci_spec_for_spawn(output);
    EXIT_OKAY
}
