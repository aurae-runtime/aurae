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
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

const PROC_SELF_EXE: &str = "/proc/self/exe";

/// Helper type to locate the auraed exe path.
#[derive(Debug, Clone)]
pub struct AuraedPath(Option<PathBuf>);

impl AuraedPath {
    /// Set an explicit path to the auraed exe
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Self {
        Self(Some(path.into()))
    }
}

impl Default for AuraedPath {
    /// Defaults to reading the symbolic link from "/proc/self/exe".
    /// During testing (i.e., `#[cfg(test)]`), the path is set to "auraed".
    fn default() -> Self {
        let path = {
            // We use `None` to delay reading the symbolic link from /proc/self/exe, as /proc may not be mounted yet
            #[cfg(not(test))]
            let path = None;

            // In unit tests, we cannot use /proc/self/exe since main.rs is not part of the test binary.
            #[cfg(test)]
            let path = Some("auraed".into());

            path
        };

        Self(path)
    }
}

impl TryFrom<AuraedPath> for PathBuf {
    type Error = std::io::Error;

    /// Can fail when relying on reading the symbolic link from /proc/self/exe, which is the default behavior.
    fn try_from(value: AuraedPath) -> Result<Self, Self::Error> {
        match value.0 {
            Some(p) => Ok(p),
            None => std::fs::read_link(PROC_SELF_EXE),
        }
    }
}

impl Display for AuraedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(path) => path.display().fmt(f),
            None => std::fmt::Display::fmt(PROC_SELF_EXE, f),
        }
    }
}