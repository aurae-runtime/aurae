/*===========================================================================*\
 *           MIT License Copyright (c) 2022 Kris Nóva <kris@nivenly.com>     *
 *                                                                           *
 *                ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓                *
 *                ┃   ███╗   ██╗ ██████╗ ██╗   ██╗ █████╗   ┃                *
 *                ┃   ████╗  ██║██╔═████╗██║   ██║██╔══██╗  ┃                *
 *                ┃   ██╔██╗ ██║██║██╔██║██║   ██║███████║  ┃                *
 *                ┃   ██║╚██╗██║████╔╝██║╚██╗ ██╔╝██╔══██║  ┃                *
 *                ┃   ██║ ╚████║╚██████╔╝ ╚████╔╝ ██║  ██║  ┃                *
 *                ┃   ╚═╝  ╚═══╝ ╚═════╝   ╚═══╝  ╚═╝  ╚═╝  ┃                *
 *                ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛                *
 *                                                                           *
 *                       This machine kills fascists.                        *
 *                                                                           *
\*===========================================================================*/

extern crate core;

use clap::*;
use log::*;
use syslog::*;

const EXIT_OKAY: i32 = 0;
//const EXIT_ERROR: i32 = 1;

fn runtime() -> i32 {
    return EXIT_OKAY;
}

fn runtime_environment() {
    let name = "nova";

    // Initialize the program
    let matches = App::new("Nova")
        .version("1.0")
        .author("Kris Nóva <kris@nivenly.com>")
        .about(name)
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .help("Toggle the verbosity bit.") // With <3 from @togglebit
                .takes_value(false),
        )
        .get_matches();

    // The logger will log to stdout and the syslog by default.
    // We hold the opinion that the program is either "verbose"
    // or it's not.
    //
    // Normal mode: Info, Warn, Error
    // Verbose mode: Debug, Trace, Info, Warn, Error
    let logger_level = if matches.is_present("verbose") {
        log::Level::Trace
    } else {
        log::Level::Info
    };

    // Syslog formatter
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: name.into(),
        pid: 0,
    };

    // Initialize the logger
    let logger_simple =
        simplelog::SimpleLogger::new(logger_level.to_level_filter(), simplelog::Config::default());
    let logger_syslog = syslog::unix(formatter).unwrap();
    let _ = match multi_log::MultiLogger::init(
        vec![logger_simple, Box::new(BasicLogger::new(logger_syslog))],
        logger_level,
    ) {
        Ok(_) => {}
        Err(e) => panic!("unable to connect to syslog: {:?}", e),
    };

    // Initialize the program
    info!("*");
    info!("*");
    info!("* Runtime environment initialized: {}", name);
    info!("*  -> Syslog process name: {}", name);
    debug!("* Runtime **debugging** enabled: {}", name);
    info!("*");
    info!("*");
}

fn main() {
    runtime_environment();
    let exit_code = runtime();
    std::process::exit(exit_code);
}
