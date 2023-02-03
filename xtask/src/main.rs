mod build_ebpf;

use std::process::exit;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    BuildEbpf(build_ebpf::Options),
}

fn main() {
    let opts = Options::parse();

    use Command::*;
    let BuildEbpf(build_opts) = opts.command;
    let ret = build_ebpf::build_ebpf(build_opts);

    if let Err(e) = ret {
        eprintln!("{e:#}");
        exit(1);
    }
}
