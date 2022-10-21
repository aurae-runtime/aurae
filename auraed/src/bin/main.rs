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
#![warn(bad_style,
        dead_code,
        improper_ctypes,
        non_shorthand_field_patterns,
        no_mangle_generic_items,
        path_statements,
        private_in_public,
        unconditional_recursion,
        unused,
        // TODO: unused_allocation,
        // TODO: unused_comparisons,
        // TODO: unused_parens,
        while_true
        )]

#![warn(// TODO: missing_copy_implementations,
        // TODO: missing_debug_implementations,
        // TODO: missing_docs,
        // TODO: trivial_casts,
        trivial_numeric_casts,
        // TODO: unused_extern_crates,
        // TODO: unused_import_braces,
        // TODO: unused_qualifications,
        // TODO: unused_results
        )]

use auraed::{logging::logchannel::LogChannel, *};
use clap::Parser;
use log::*;
use std::path::PathBuf;

const EXIT_OKAY: i32 = 0;
const EXIT_ERROR: i32 = 1;

/// Command line options for auraed.
///
/// Defines the configurable options which can be used to populate
/// an AuraeRuntime structure.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct AuraedOptions {
    /// The signed server certificate. Defaults to /etc/aurae/pki/_signed.server.crt
    #[clap(
        long,
        value_parser,
        default_value = "/etc/aurae/pki/_signed.server.crt"
    )]
    server_crt: String,
    /// The secret server key. Defaults to /etc/aurae/pki/server.key
    #[clap(long, value_parser, default_value = "/etc/aurae/pki/server.key")]
    server_key: String,
    /// The CA certificate. Defaults to /etc/aurae/pki/ca.crt
    #[clap(long, value_parser, default_value = "/etc/aurae/pki/ca.crt")]
    ca_crt: String,
    /// Aurae socket path. Defaults to /var/run/aurae/aurae.sock
    #[clap(short, long, value_parser, default_value = auraed::AURAE_SOCK)]
    socket: String,
    /// Toggle verbosity. Default false
    #[clap(short, long)]
    verbose: bool,
}

async fn daemon() -> i32 {
    let options = AuraedOptions::parse();

    // The logger will log to stdout and the syslog by default.
    // We hold the opinion that the program is either "verbose"
    // or it's not.
    //
    // Normal mode: Info, Warn, Error
    // Verbose mode: Debug, Trace, Info, Warn, Error
    // let logger_level = if matches.is_present("verbose") {
    let logger_level = if options.verbose { Level::Trace } else { Level::Info };

    let log_collector = LogChannel::new("AuraeRuntime");
    // Log Collector used to expose logs via API
    let prod = log_collector.get_producer();

    // Initializes Logging and prepares system if auraed is run as pid=1
    init::init(logger_level, prod).await;

    trace!("**Logging: Verbose Mode**");
    info!("Starting Aurae Daemon Runtime...");

    let runtime = AuraedRuntime {
        server_crt: PathBuf::from(options.server_crt),
        server_key: PathBuf::from(options.server_key),
        ca_crt: PathBuf::from(options.ca_crt),
        socket: PathBuf::from(options.socket),
        log_collector,
    };

    let e = runtime.run().await;
    if e.is_err() {
        error!("{:?}", e);
    }

    if e.is_err() {
        EXIT_ERROR
    } else {
        EXIT_OKAY
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exit_code = daemon();
    std::process::exit(exit_code.await);
}
