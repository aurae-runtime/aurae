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
use client::cells::cell_service::CellServiceClient;
use test_helpers::*;

mod common;

#[test_helpers_macros::shared_runtime_test]
async fn cells_start_stop_delete() {
    skip_if_not_root!("cells_start_stop_delete");
    skip_if_seccomp!("cells_start_stop_delete");

    let client = common::auraed_client().await;

    // Allocate a cell
    let cell_name = retry!(
        client
            .allocate(
                common::cells::CellServiceAllocateRequestBuilder::new().build()
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // Start the executable
    let req = common::cells::CellServiceStartRequestBuilder::new()
        .cell_name(cell_name.clone())
        .executable_name("aurae-exe".to_string())
        .build();
    let _ = retry!(client.start(req.clone()).await).unwrap().into_inner();

    // Stop the executable
    let _ = retry!(
        client
            .stop(proto::cells::CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: "aurae-exe".to_string(),
            })
            .await
    )
    .unwrap();

    // Delete the cell
    let _ = retry!(
        client
            .free(proto::cells::CellServiceFreeRequest {
                cell_name: cell_name.clone()
            })
            .await
    )
    .unwrap();
}

/// Regression test for https://github.com/aurae-runtime/aurae/issues/534.
///
/// Runs a command that spawns two background children of a `bash` wrapper
/// (the original bug: the `sh -c`/`bash -c` wrapper is tracked but its
/// children leak when stopped). After stop, the wrapper PID and both child
/// PIDs must all be gone from /proc.
#[test_helpers_macros::shared_runtime_test]
async fn cells_stop_kills_entire_process_group() {
    skip_if_not_root!("cells_stop_kills_entire_process_group");
    skip_if_seccomp!("cells_stop_kills_entire_process_group");

    let client = common::auraed_client().await;

    let cell_name = retry!(
        client
            .allocate(
                common::cells::CellServiceAllocateRequestBuilder::new().build()
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    // auraed wraps the supplied command as `sh -c <cmd>` (see the
    // ValidatedExecutable -> ExecutableSpec conversion in validation.rs),
    // so the recorded leader pid is the `sh` process. `process_group(0)`
    // makes that leader its own pgid leader; `bash` and the two `sleep`
    // children inherit the pgid. We therefore expect at least 2 group
    // members under the leader after start.
    let req = common::cells::CellServiceStartRequestBuilder::new()
        .cell_name(cell_name.clone())
        .executable_name("group-leaker".to_string())
        .command("bash -c 'sleep 9000 & sleep 9000 & wait'".to_string())
        .build();
    let start = retry!(client.start(req.clone()).await).unwrap().into_inner();
    let leader_pid = start.pid;
    assert!(leader_pid > 0, "expected a valid leader pid");

    // Wait for bash to fork its two sleep children rather than guessing a
    // fixed sleep — under load the fork can take longer than a few hundred
    // millis, which would flake on a fixed delay.
    let spawned = wait_until(std::time::Duration::from_secs(2), || {
        children_by_pgid(leader_pid).len() >= 2
    })
    .await;
    assert!(spawned, "bash did not spawn its sleep children within 2s");

    let children_before = children_by_pgid(leader_pid);
    assert!(
        children_before.len() >= 2,
        "expected leader to have spawned >= 2 children in pgid {leader_pid}, \
         found {:?}",
        children_before
    );

    let _ = retry!(
        client
            .stop(proto::cells::CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: "group-leaker".to_string(),
            })
            .await
    )
    .unwrap();

    // After stop returns, every PID in the group must be gone. Poll for a
    // short window to tolerate reaping latency.
    let all_gone = wait_until(std::time::Duration::from_secs(3), || {
        !pid_exists(leader_pid)
            && children_before.iter().all(|pid| !pid_exists(*pid))
    })
    .await;
    assert!(
        all_gone,
        "orphans remain after stop: leader {}={}, children={:?}",
        leader_pid,
        pid_exists(leader_pid),
        children_before
            .iter()
            .map(|p| (*p, pid_exists(*p)))
            .collect::<Vec<_>>()
    );

    let _ = retry!(
        client
            .free(proto::cells::CellServiceFreeRequest {
                cell_name: cell_name.clone()
            })
            .await
    )
    .unwrap();
}

/// Calling stop twice on the same executable should return Ok both times.
#[test_helpers_macros::shared_runtime_test]
async fn cells_double_stop_is_idempotent() {
    skip_if_not_root!("cells_double_stop_is_idempotent");
    skip_if_seccomp!("cells_double_stop_is_idempotent");

    let client = common::auraed_client().await;

    let cell_name = retry!(
        client
            .allocate(
                common::cells::CellServiceAllocateRequestBuilder::new().build()
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    let req = common::cells::CellServiceStartRequestBuilder::new()
        .cell_name(cell_name.clone())
        .executable_name("double-stopper".to_string())
        .command("sleep 9000".to_string())
        .build();
    let _ = retry!(client.start(req.clone()).await).unwrap().into_inner();

    let _ = retry!(
        client
            .stop(proto::cells::CellServiceStopRequest {
                cell_name: Some(cell_name.clone()),
                executable_name: "double-stopper".to_string(),
            })
            .await
    )
    .expect("first stop should succeed");

    let second = client
        .stop(proto::cells::CellServiceStopRequest {
            cell_name: Some(cell_name.clone()),
            executable_name: "double-stopper".to_string(),
        })
        .await;
    assert!(
        second.is_ok(),
        "second stop should be idempotent; got {:?}",
        second.err()
    );

    let _ = retry!(
        client
            .free(proto::cells::CellServiceFreeRequest {
                cell_name: cell_name.clone()
            })
            .await
    )
    .unwrap();
}

/// Stopping an executable that has already exited on its own must still
/// return Ok. This drives the ESRCH/ECHILD path in `Executables::stop` —
/// a refactor that moves those errno values into the FailedToStopExecutable
/// branch would silently turn natural-exit stops into Status::internal.
#[test_helpers_macros::shared_runtime_test]
async fn cells_stop_after_natural_exit_is_ok() {
    skip_if_not_root!("cells_stop_after_natural_exit_is_ok");
    skip_if_seccomp!("cells_stop_after_natural_exit_is_ok");

    let client = common::auraed_client().await;

    let cell_name = retry!(
        client
            .allocate(
                common::cells::CellServiceAllocateRequestBuilder::new().build()
            )
            .await
    )
    .unwrap()
    .into_inner()
    .cell_name;

    let req = common::cells::CellServiceStartRequestBuilder::new()
        .cell_name(cell_name.clone())
        .executable_name("self-exit".to_string())
        .command("true".to_string())
        .build();
    let start = retry!(client.start(req.clone()).await).unwrap().into_inner();
    let leader_pid = start.pid;

    // Wait for the process to disappear from /proc on its own.
    let exited = wait_until(std::time::Duration::from_secs(5), || {
        !pid_exists(leader_pid)
    })
    .await;
    assert!(exited, "expected leader pid {leader_pid} to exit on its own");

    let stop = client
        .stop(proto::cells::CellServiceStopRequest {
            cell_name: Some(cell_name.clone()),
            executable_name: "self-exit".to_string(),
        })
        .await;
    assert!(
        stop.is_ok(),
        "stop after natural exit should be idempotent; got {:?}",
        stop.err()
    );

    let _ = retry!(
        client
            .free(proto::cells::CellServiceFreeRequest {
                cell_name: cell_name.clone()
            })
            .await
    )
    .unwrap();
}

fn pid_exists(pid: i32) -> bool {
    std::fs::metadata(format!("/proc/{pid}")).is_ok()
}

/// Enumerate /proc and return pids whose PGID equals `pgid`.
/// Returns pids other than `pgid` itself (i.e. the leader's group members).
fn children_by_pgid(pgid: i32) -> Vec<i32> {
    let mut pids = Vec::new();
    let entries = match std::fs::read_dir("/proc") {
        Ok(e) => e,
        Err(_) => return pids,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(s) = name.to_str() else { continue };
        let Ok(pid) = s.parse::<i32>() else { continue };
        if pid == pgid {
            continue;
        }
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat"))
        else {
            continue;
        };
        // The /proc/<pid>/stat comm field is wrapped in `(...)` and may
        // contain spaces/parens; rsplit on the last `)` to skip past it
        // safely. Fields beyond `)` are space-separated and ordered:
        // [0]=state, [1]=ppid, [2]=pgrp, [3]=session, ...
        let Some(after_comm) = stat.rsplit_once(')').map(|x| x.1) else {
            continue;
        };
        let fields: Vec<&str> = after_comm.split_whitespace().collect();
        if let Some(pgrp_str) = fields.get(2)
            && let Ok(pgrp) = pgrp_str.parse::<i32>()
            && pgrp == pgid
        {
            pids.push(pid);
        }
    }
    pids
}

async fn wait_until<F: Fn() -> bool>(
    timeout: std::time::Duration,
    check: F,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if check() {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    check()
}
