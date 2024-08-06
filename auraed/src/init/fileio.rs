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

use std::path::Path;

use anyhow::anyhow;
use walkdir::WalkDir;

#[allow(unused)]
pub(crate) fn show_dir(
    dir: impl AsRef<Path>,
    recurse: bool,
) -> anyhow::Result<()> {
    let mut dir = WalkDir::new(dir);
    if !recurse {
        dir = dir.max_depth(0);
    }

    for entry in dir {
        match entry {
            Ok(p) => println!("{}", p.path().display()),
            Err(e) => {
                return if let Some(path) = e.path() {
                    Err(anyhow!(
                        "Error reading directory. Could not read path {}. Error: {}",
                        path.display(),
                        e
                    ))
                } else {
                    Err(anyhow!("Error reading directory. Error: {}", e))
                }
            }
        }
    }

    Ok(())
}