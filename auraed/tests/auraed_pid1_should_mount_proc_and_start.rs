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

//! Ensure auraed can run as PID1, mounts /proc, and serves its Unix socket.

mod common;
use crate::common::tls::{TlsMaterial, generate_server_tls};

use std::{
    io,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use test_helpers::*;

#[test]
fn auraed_pid1_should_mount_proc_and_start() {
    skip_if_not_root!("auraed_pid1_should_mount_proc_and_start");
    skip_if_seccomp!("auraed_pid1_should_mount_proc_and_start");

    let tempdir = tempfile::tempdir().expect("tempdir");
    let runtime_dir = tempdir.path().join("runtime");
    let library_dir = tempdir.path().join("library");
    std::fs::create_dir_all(&runtime_dir).expect("runtime dir");
    std::fs::create_dir_all(&library_dir).expect("library dir");

    let log_path = tempdir.path().join("auraed.log");
    let tls = generate_server_tls(tempdir.path());

    let child =
        match spawn_auraed_pid1(&runtime_dir, &library_dir, &log_path, &tls) {
            Ok(child) => child,
            Err(e)
                if e.kind() == io::ErrorKind::NotFound
                    || e.kind() == io::ErrorKind::PermissionDenied =>
            {
                eprintln!("unshare not available: {e}; skipping");
                return;
            }
            Err(e) => panic!("failed to spawn auraed as pid1: {e}"),
        };
    let Some(child) = child else { return };
    let auraed_pid = wait_for_pid_file(tempdir.path(), Duration::from_secs(5));
    let _guard = common::ChildGuard::new(child);

    if !wait_for_tcp_listener(auraed_pid, &log_path, Duration::from_secs(20)) {
        return;
    }

    assert_ns_pid1(auraed_pid);
    assert!(
        mount_present(auraed_pid, "/proc"),
        "expected /proc to be mounted for auraed as pid1 (pid={auraed_pid})"
    );
    assert!(
        mount_present(auraed_pid, "/sys/fs/cgroup"),
        "expected cgroup2 to be mounted for auraed as pid1 (pid={auraed_pid})"
    );

    eprintln!("pid1 test complete: auraed pid {auraed_pid}");
}

fn spawn_auraed_pid1(
    runtime_dir: &Path,
    library_dir: &Path,
    log_path: &Path,
    tls: &TlsMaterial,
) -> Result<Option<Child>, io::Error> {
    // Use unshare to create a new PID and mount namespace where auraed becomes PID1 and mounts /proc.
    let log =
        std::fs::OpenOptions::new().create(true).append(true).open(log_path)?;

    // Copy auraed binary into the temp runtime so it's executable inside the user namespace.
    let auraed_bin = runtime_dir.join("auraed-bin");
    std::fs::copy(env!("CARGO_BIN_EXE_auraed"), &auraed_bin)
        .expect("copy auraed bin");
    std::fs::set_permissions(
        &auraed_bin,
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("chmod auraed bin");

    let pidfile = runtime_dir.join("auraed.pid");

    // `--fork --pid` makes the command PID1 inside the new namespace.
    let child = Command::new("unshare")
        .arg("-p")
        .arg("-m")
        .arg("-n")
        .arg("--fork")
        .arg("--mount-proc")
        .arg("sh")
        .arg("-c")
        .arg(format!(
            // We take the host auraed binary (copied to the temp runtime) and exec it as PID1
            // in a new PID+mount+net namespace. Because cgroup2 cannot be mounted from an
            // unprivileged namespace, we rbind the host cgroup2 mount into the new namespace
            // so auraed can initialize its cgroups. /dev and /run are tmpfs to avoid polluting
            // the host; we also stub /dev/input/event0 for the power button listener.
            "set -ex; echo $$ > {pidfile}; mount --make-rprivate /; mount -t tmpfs tmpfs /dev; mkdir -p /dev/input; : > /dev/input/event0; mount -t tmpfs tmpfs /run; mkdir -p /sys /sys/fs/cgroup; mount --rbind /sys/fs/cgroup /sys/fs/cgroup || true; ip link set lo up; ip link add eth0 type dummy; ip link set eth0 up; exec {bin} --runtime-dir {rt} --library-dir {lib} --ca-crt {ca} --server-crt {crt} --server-key {key}",
            pidfile = pidfile.display(),
            bin = auraed_bin.display(),
            rt = runtime_dir.display(),
            lib = library_dir.display(),
            ca = tls.ca_crt.display(),
            crt = tls.server_crt.display(),
            key = tls.server_key.display(),
        ))
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()?;

    Ok(Some(child))
}

fn wait_for_tcp_listener(pid: u32, log_path: &Path, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let logs =
            std::fs::read_to_string(log_path).unwrap_or_else(|_| String::new());
        if logs.contains("TCP Access Socket created") {
            return true;
        }
        if !std::fs::metadata(format!("/proc/{pid}")).is_ok() {
            panic!("auraed pid {pid} exited early. logs:\n{}", logs);
        }
        thread::sleep(Duration::from_millis(50));
    }
    let logs = std::fs::read_to_string(log_path)
        .unwrap_or_else(|_| "<unavailable>".to_string());
    panic!("TCP listener not reported within {:?}. logs:\n{}", timeout, logs);
}

fn wait_for_pid_file(dir: &Path, timeout: Duration) -> u32 {
    let pidfile = dir.join("runtime").join("auraed.pid");
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(contents) = std::fs::read_to_string(&pidfile) {
            if let Ok(pid) = contents.trim().parse::<u32>() {
                return pid;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("pidfile {:?} not created within {:?}", pidfile, timeout);
}

fn assert_ns_pid1(pid: u32) {
    let status_path = format!("/proc/{pid}/status");
    let status =
        std::fs::read_to_string(&status_path).unwrap_or_else(|_| String::new());
    if status.is_empty() {
        panic!("status file {status_path} empty or missing");
    }
    if !std::fs::metadata(format!("/proc/{pid}")).is_ok() {
        panic!("process {pid} not alive");
    }
    if let Some(ns_line) = status.lines().find(|l| l.starts_with("NSpid:")) {
        if ns_line.split_whitespace().last() != Some("1") {
            panic!(
                "expected auraed to be pid 1 in its namespace, got {ns_line}"
            );
        }
    }
    // If NSpid missing, we accept the process as long as it is alive; some kernels omit NSpid.
}

fn mount_present(pid: u32, target: &str) -> bool {
    let Ok(mountinfo) =
        std::fs::read_to_string(format!("/proc/{pid}/mountinfo"))
    else {
        return false;
    };
    mountinfo
        .lines()
        .filter_map(|line| line.split_whitespace().nth(4))
        .any(|mount_point| mount_point == target)
}
