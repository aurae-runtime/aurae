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
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    generate_aurae_ts();
}

fn generate_aurae_ts() {
    // Currently nothing is generated.
    // We are only copying the contents of helpers.ts to the gen directory.
    // If we do generate code in the future, we won't need to change all the imports.

    let aurae = include_str!("./aurae.ts");

    let gen_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(out_dir) => {
            let mut out_dir = PathBuf::from(out_dir);
            out_dir.push("gen");
            out_dir
        }
        _ => PathBuf::from("gen"),
    };

    let ts_path = {
        let mut out_dir = gen_dir;
        out_dir.push("aurae.ts");
        out_dir
    };

    let mut ts = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ts_path.clone())
        .unwrap_or_else(|_| {
            panic!("Failed to create or overwrite {ts_path:?}")
        });

    write!(ts, "{aurae}")
        .unwrap_or_else(|_| panic!("Could not write to {ts_path:?}"));
}