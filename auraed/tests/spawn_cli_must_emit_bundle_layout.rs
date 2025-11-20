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

use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

#[test]
fn spawn_cli_must_emit_bundle_layout() {
    let tempdir = tempfile::tempdir().expect("create temp dir");
    let bundle_dir = tempdir.path().join("bundle");

    let status = std::process::Command::new(env!("CARGO_BIN_EXE_auraed"))
        .arg("spawn")
        .arg("--output")
        .arg(bundle_dir.to_str().expect("bundle path"))
        .status()
        .expect("failed to run auraed spawn CLI");

    assert!(
        status.success(),
        "auraed spawn command should succeed, got status {status:?}"
    );

    assert_file(&bundle_dir.join("config.json"));
    let auraed_bin = bundle_dir.join("rootfs/bin/auraed");
    assert_file(&auraed_bin);
    let init_link = bundle_dir.join("rootfs/bin/init");
    assert_file(&init_link);

    let auraed_meta =
        std::fs::metadata(&auraed_bin).expect("metadata for auraed binary");
    let init_meta =
        std::fs::metadata(&init_link).expect("metadata for init binary");
    assert_eq!(
        (auraed_meta.dev(), auraed_meta.ino()),
        (init_meta.dev(), init_meta.ino()),
        "init should be a hard link to auraed binary"
    );
}

fn assert_file(path: &PathBuf) {
    assert!(path.is_file(), "expected {:?} to be a regular file", path);
}
