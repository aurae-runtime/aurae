/* -------------------------------------------------------------------------- *\
 *        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

use super::error::{Result, RuntimeServiceError};

use libcontainer::container::Container;
use std::collections::HashMap;

type Cache = HashMap<String, Container>;

#[derive(Debug, Clone, Default)]
pub struct SandboxCache {
    cache: Cache,
}

impl SandboxCache {
    pub fn add(
        &mut self,
        sandbox_id: String,
        sandbox: Container,
    ) -> Result<()> {
        if self.cache.contains_key(&sandbox_id) {
            return Err(RuntimeServiceError::SandboxExists { sandbox_id });
        }
        let _ = self.cache.insert(sandbox_id, sandbox);
        Ok(())
    }

    pub fn get_mut(&mut self, sandbox_id: &String) -> Result<&mut Container> {
        let Some(sandbox) = self.cache.get_mut(sandbox_id) else {
                return Err(RuntimeServiceError::SandboxNotFound { sandbox_id: sandbox_id.clone() });
            };
        Ok(sandbox)
    }

    pub fn get(&self, sandbox_id: &String) -> Result<&Container> {
        let Some(sandbox) = self.cache.get(sandbox_id) else {
                return Err(RuntimeServiceError::SandboxNotFound { sandbox_id: sandbox_id.clone() });
            };
        Ok(sandbox)
    }

    pub fn list(&self) -> Result<Vec<&Container>> {
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
