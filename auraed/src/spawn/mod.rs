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

use anyhow::Context;
use std::fs;
use std::fs::Permissions;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};

const PROC_SELF_EXE: &str = "/proc/self/exe";
//const SPAWN_CONFIG: &[u8] = include_bytes!("config.json");

// TODO accept a OCI config from calling code (CRI has linux config that will need to be mapped to spec)
pub fn spawn_auraed_oci_to(
    output: PathBuf,
    spec: oci_spec::runtime::Spec,
) -> Result<(), anyhow::Error> {
    // Here we read /proc/self/exe which will be a symbolic link to our binary.
    let auraed_path = fs::read_link(PROC_SELF_EXE)
        .context("reading auraed sym link from procfs")?;
    let binary_data = fs::read(auraed_path)
        .context("reading auraed executable from procfs")?;

    // Reset the output directory (if exists)
    let _ = fs::remove_dir_all(&output).context("remove output dir clean");
    fs::create_dir_all(&output).context("create new output dir clean")?;

    // Write our config.json
    let config_contents =
        serde_json::to_vec_pretty(&spec).expect("json serialize oci config");

    fs::write(output.join(Path::new("config.json")), config_contents)
        .expect("writing default config.json for spawn image");

    // .
    // ├── config.json
    // └── rootfs
    //     └── bin
    //         ├── auraed
    //         └── init

    // Create the initial file structure for the Auraed OCI container
    //
    // bin    etc    lib    media  opt    root   sbin   sys    usr
    // dev    home   lib64  mnt    proc   run    srv    tmp    var
    fs::create_dir_all(output.join(Path::new("rootfs")))?;
    fs::create_dir_all(output.join(Path::new("rootfs/bin")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/lib")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/media")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/opt")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/root")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/sbin")))?;
    fs::create_dir_all(output.join(Path::new("rootfs/sys")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/usr")))?;
    fs::create_dir_all(output.join(Path::new("rootfs/dev")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/home")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/lib64")))?;
    fs::create_dir_all(output.join(Path::new("rootfs/mnt")))?;
    fs::create_dir_all(output.join(Path::new("rootfs/proc")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/run")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/srv")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/tmp")))?;
    //fs::create_dir_all(output.join(Path::new("rootfs/var")))?;

    // /bin/auraed
    fs::write(
        Path::new(&output.join(Path::new("rootfs/bin/auraed"))),
        binary_data,
    )?;
    fs::set_permissions(
        Path::new(&output.join(Path::new("rootfs/bin/auraed"))),
        Permissions::from_mode(0o755),
    )?;

    fs::hard_link(
        output.join(Path::new("rootfs/bin/auraed")),
        output.join(Path::new("rootfs/bin/init")),
    )
    .expect("linking /bin/auraed to /bin/init");

    Ok(())
}