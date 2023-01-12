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

use std::fs;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

const PROC_SELF_EXE: &str = "/proc/self/exe";
const SPAWN_OCI_NAME: &str = "aurae-spawn";

pub fn spawn_auraed_oci_to(output: &str) -> Result<(), anyhow::Error> {
    // Here we read /proc/self/exe which will be a symbolic link to our binary.
    let auraed_path = fs::read_link(PROC_SELF_EXE)?;
    let binary_data = fs::read(auraed_path)?;
    let oci_bundle_path = Path::new(output).join(Path::new(SPAWN_OCI_NAME));

    // Reset the output directory
    fs::remove_dir_all(oci_bundle_path.clone())?;

    // .
    // ├── config.json
    // └── rootfs
    //     └── bin
    //         └── auraed

    // Create the initial file structure for the Auraed OCI container
    //
    // bin    etc    lib    media  opt    root   sbin   sys    usr
    // dev    home   lib64  mnt    proc   run    srv    tmp    var
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/bin")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/lib")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/media")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/opt")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/root")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/sbin")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/sys")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/usr")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/dev")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/home")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/lib64")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/mnt")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/proc")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/run")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/srv")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/tmp")))?;
    fs::create_dir_all(oci_bundle_path.join(Path::new("rootfs/var")))?;

    // /bin/auraed
    fs::write(
        Path::new(&oci_bundle_path.join(Path::new("rootfs/bin/auraed"))),
        binary_data,
    )?;
    fs::set_permissions(
        Path::new(&oci_bundle_path.join(Path::new("rootfs/bin/auraed"))),
        Permissions::from_mode(0o755),
    )?;

    Ok(())
}
