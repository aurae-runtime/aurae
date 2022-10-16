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

use std::path::Path;

use anyhow::anyhow;
use walkdir::WalkDir;

#[allow(dead_code)]
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
