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

use super::{
    Executable, ExecutableName, ExecutableSpec, ExecutablesError, Result,
};
use std::collections::HashMap;
use std::process::ExitStatus;

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
        // `or_insert` will always insert as we've already assured ourselves that the key does not exist.
        let executable = self
            .cache
            .entry(executable_name.clone())
            .or_insert_with(|| Executable::new(executable_spec));

        // TODO: if we fail to start, the exe remains in the cache and start cannot be called again
        // solving ^^ was a borrow checker fight and I (future-highway) lost this round.
        executable.start().map_err(|e| {
            ExecutablesError::FailedToStartExecutable {
                executable_name,
                source: e,
            }
        })?;

        Ok(executable)
    }

    pub fn stop(
        &mut self,
        executable_name: &ExecutableName,
    ) -> Result<ExitStatus> {
        let exit_status = self.get_mut(executable_name, |executable| {
            executable.kill().map_err(|e| {
                ExecutablesError::FailedToStopExecutable {
                    executable_name: executable_name.clone(),
                    source: e,
                }
            })
        })?;

        let Some(exit_status) = exit_status else {
            // Exes that never started return None
            let _ = self.cache.remove(executable_name).expect("exe in cache");
            return Err(ExecutablesError::ExecutableNotFound {
                executable_name: executable_name.clone(),
            });
        };

        let _ = self.cache.remove(executable_name).ok_or_else(|| {
            // get_mut would have already thrown this error, so we should never reach here
            ExecutablesError::ExecutableNotFound {
                executable_name: executable_name.clone(),
            }
        })?;

        Ok(exit_status)
    }

    fn get_mut<F, R>(
        &mut self,
        executable_name: &ExecutableName,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut Executable) -> Result<R>,
    {
        let Some(executable) = self.cache.get_mut(executable_name) else {
            return Err(ExecutablesError::ExecutableNotFound { executable_name: executable_name.clone() });
        };

        f(executable)
    }
}
