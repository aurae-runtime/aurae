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

use super::{
    Executable, ExecutableName, ExecutableSpec, ExecutablesError, Result,
};
use nix::libc;
use std::{collections::HashMap, process::ExitStatus};
use tracing::warn;

type Cache = HashMap<ExecutableName, Executable>;

/// An in-memory store for the list of executables created with Aurae.
#[derive(Debug, Default)]
pub struct Executables {
    cache: Cache,
}

impl Executables {
    pub fn start<T: Into<ExecutableSpec>>(
        &mut self,
        executable_spec: T,
        uid: Option<u32>,
        gid: Option<u32>,
    ) -> Result<&Executable> {
        let executable_spec = executable_spec.into();

        // TODO: replace with try_insert when it becomes stable
        // Check if there was already an executable with the same name.
        if self.cache.contains_key(&executable_spec.name) {
            return Err(ExecutablesError::ExecutableExists {
                executable_name: executable_spec.name,
            });
        }

        let executable_name = executable_spec.name.clone();
        let mut executable = Executable::new(executable_spec);

        // start the exe before we add it to the cache, as otherwise a failure leads to the
        // executable remaining in the cache and start cannot be called again.
        executable.start(uid, gid).map_err(|e| {
            ExecutablesError::FailedToStartExecutable {
                executable_name: executable_name.clone(),
                source: e,
            }
        })?;

        // `or_insert` will always insert as we've already assured ourselves that the key does not
        // exist.
        let inserted_executable =
            self.cache.entry(executable_name).or_insert_with(|| executable);

        Ok(inserted_executable)
    }

    pub fn get(&self, executable_name: &ExecutableName) -> Result<&Executable> {
        let Some(executable) = self.cache.get(executable_name) else {
            return Err(ExecutablesError::ExecutableNotFound {
                executable_name: executable_name.clone(),
            });
        };
        Ok(executable)
    }

    pub async fn stop(
        &mut self,
        executable_name: &ExecutableName,
    ) -> Result<ExitStatus> {
        let Some(executable) = self.cache.get_mut(executable_name) else {
            return Err(ExecutablesError::ExecutableNotFound {
                executable_name: executable_name.clone(),
            });
        };

        match executable.kill().await {
            Ok(Some(status)) => {
                let _ = self.cache.remove(executable_name);
                Ok(status)
            }
            Ok(None) => {
                // Cache invariant: only started executables are inserted
                // into the cache (see `start` above), so kill() on a cached
                // entry cannot return Ok(None).
                unreachable!(
                    "executable {executable_name:?} is in cache without \
                     having been started"
                );
            }
            Err(e)
                if matches!(
                    e.raw_os_error(),
                    Some(libc::ESRCH) | Some(libc::ECHILD)
                ) =>
            {
                // killpg ESRCH (group already empty) or wait ECHILD (kernel
                // already reaped). Process is gone; evict and report
                // distinctly so callers can render stop idempotent without
                // collapsing this with "name not in cache".
                warn!(
                    "executable {executable_name:?} already exited before \
                     stop: {e}"
                );
                let _ = self.cache.remove(executable_name);
                Err(ExecutablesError::ExecutableAlreadyExited {
                    executable_name: executable_name.clone(),
                })
            }
            Err(e) => Err(ExecutablesError::FailedToStopExecutable {
                executable_name: executable_name.clone(),
                source: e,
            }),
        }
    }

    /// Stops all executables concurrently
    pub async fn broadcast_stop(&mut self) {
        let mut names = vec![];
        for exe in self.cache.values_mut() {
            if let Err(e) = exe.kill().await {
                warn!("broadcast_stop: failed to kill {:?}: {e}", exe.name);
            }
            names.push(exe.name.clone())
        }

        for name in names {
            let _ = self.cache.remove(&name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    use tokio::process::Command;

    fn spec_for(name: &ExecutableName) -> ExecutableSpec {
        spec_with_command(name, "sleep 60")
    }

    fn spec_with_command(
        name: &ExecutableName,
        sh_arg: &str,
    ) -> ExecutableSpec {
        let mut command = Command::new("sh");
        let _ = command.arg("-c");
        let _ = command.arg(sh_arg);
        ExecutableSpec {
            name: name.clone(),
            description: format!("test executable {name}"),
            command,
        }
    }

    #[tokio::test]
    async fn start_should_cache_pid_and_reject_duplicates() {
        let mut executables = Executables::default();
        let exe_name = ExecutableName::new(format!(
            "unit-test-exe-{}",
            uuid::Uuid::new_v4()
        ));

        let executable = executables
            .start(spec_for(&exe_name), None, None)
            .expect("start executable");
        assert!(
            executable.pid().is_some(),
            "expected spawned process to expose a pid"
        );

        let err = executables
            .start(spec_for(&exe_name), None, None)
            .expect_err("duplicate start should fail");
        assert!(
            matches!(err, ExecutablesError::ExecutableExists { .. }),
            "expected ExecutableExists error, got {err:?}"
        );

        let status =
            executables.stop(&exe_name).await.expect("stop executable");
        assert!(
            status.success() || status.signal() == Some(9),
            "expected graceful stop or SIGKILL, got status {status:?}"
        );
    }

    /// Stopping a short-lived executable that has already finished running
    /// must still return Ok (the cache holds the Stopped state) and must
    /// evict the cache entry.
    #[tokio::test]
    async fn stop_after_natural_exit_returns_ok_and_evicts() {
        let mut executables = Executables::default();
        let exe_name = ExecutableName::new(format!(
            "unit-test-self-exit-{}",
            uuid::Uuid::new_v4()
        ));

        let pid = executables
            .start(spec_with_command(&exe_name, "true"), None, None)
            .expect("start executable")
            .pid()
            .expect("captured pid")
            .as_raw();

        // Give the leader time to exit. It will sit as a zombie until
        // child.wait() is called inside stop(); we just need to ensure the
        // process has actually finished its work before we test stop().
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::fs::metadata(format!("/proc/{pid}/cmdline"))
            .map(|_| true)
            .unwrap_or(false)
            && std::time::Instant::now() < deadline
        {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        let _ = executables
            .stop(&exe_name)
            .await
            .expect("stop after natural exit should be Ok");

        // Cache must have been evicted, so a second stop reports
        // ExecutableNotFound (the cache-miss variant), distinct from
        // ExecutableAlreadyExited.
        let err = executables
            .stop(&exe_name)
            .await
            .expect_err("second stop should report ExecutableNotFound");
        assert!(
            matches!(err, ExecutablesError::ExecutableNotFound { .. }),
            "expected ExecutableNotFound after eviction, got {err:?}"
        );
    }

    /// Stopping a name that was never inserted must return ExecutableNotFound,
    /// not the already-exited variant.
    #[tokio::test]
    async fn stop_unknown_name_returns_not_found() {
        let mut executables = Executables::default();
        let exe_name = ExecutableName::new("never-started".to_string());

        let err = executables
            .stop(&exe_name)
            .await
            .expect_err("stop on unknown name should fail");
        assert!(
            matches!(err, ExecutablesError::ExecutableNotFound { .. }),
            "expected ExecutableNotFound, got {err:?}"
        );
    }
}
