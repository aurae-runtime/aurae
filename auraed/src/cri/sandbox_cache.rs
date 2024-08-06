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

use super::error::{Result, RuntimeServiceError};
use crate::cri::sandbox::Sandbox;
use std::collections::HashMap;

/// Cache is the in-memory cache which is embedded
/// into the SandboxCache structure which provides access
/// and controls for the cache.
type Cache = HashMap<String, Sandbox>;

#[derive(Debug, Clone, Default)]
pub struct SandboxCache {
    cache: Cache,
}

impl SandboxCache {
    pub fn add(&mut self, sandbox_id: String, sandbox: Sandbox) -> Result<()> {
        if self.cache.contains_key(&sandbox_id) {
            return Err(RuntimeServiceError::SandboxExists { sandbox_id });
        }
        let _ = self.cache.insert(sandbox_id, sandbox);
        Ok(())
    }

    pub fn get_mut(&mut self, sandbox_id: &String) -> Result<&mut Sandbox> {
        let Some(sandbox) = self.cache.get_mut(sandbox_id) else {
                return Err(RuntimeServiceError::SandboxNotFound { sandbox_id: sandbox_id.clone() });
            };
        Ok(sandbox)
    }

    pub fn get(&self, sandbox_id: &String) -> Result<&Sandbox> {
        let Some(sandbox) = self.cache.get(sandbox_id) else {
                return Err(RuntimeServiceError::SandboxNotFound { sandbox_id: sandbox_id.clone() });
            };
        Ok(sandbox)
    }

    pub fn list(&self) -> Result<Vec<&Sandbox>> {
        Ok(self.cache.values().collect())
    }

    pub fn remove(&mut self, sandbox_id: &String) -> Result<()> {
        if self.cache.remove(sandbox_id).is_none() {
            return Err(RuntimeServiceError::SandboxNotFound {
                sandbox_id: sandbox_id.clone(),
            });
        }
        Ok(())
    }
}