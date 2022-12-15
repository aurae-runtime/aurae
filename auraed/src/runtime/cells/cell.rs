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

use super::Result;
use crate::runtime::cells::{
    validation::ValidatedCell, CellName, CellsError, Executable, ExecutableName,
};
use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::{hierarchies, Cgroup, Hierarchy};
use log::info;
use std::collections::HashMap;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub(crate) struct Cell {
    name: CellName,
    cgroup: Cgroup,
    executables: HashMap<ExecutableName, Executable>,
}

impl Cell {
    // TODO: This fn signature ties cells module to runtime module (refactor to better solution)
    // Here is where we define the "default" cgroup parameters for Aurae cells
    pub fn allocate(cell_spec: ValidatedCell) -> Self {
        let ValidatedCell { name, cpu_cpus, cpu_shares, cpu_mems, cpu_quota } =
            cell_spec;

        let hierarchy = hierarchy();
        let cgroup: Cgroup = CgroupBuilder::new(&name)
            // CPU Controller
            .cpu()
            .shares(cpu_shares.into_inner())
            .mems(cpu_mems.into_inner())
            .period(1000000) // microseconds in a second
            .quota(cpu_quota.into_inner())
            .cpus(cpu_cpus.into_inner())
            .done()
            // Final Build
            .build(hierarchy);

        Self { name, cgroup, executables: Default::default() }
    }

    pub fn free(self) -> Result<()> {
        self.cgroup.delete()?;
        Ok(())
    }

    pub fn spawn_executable(
        &mut self,
        exe_name: ExecutableName,
        mut command: Command,
        args: Vec<String>,
        _description: String,
    ) -> Result<()> {
        // Check if there was already an executable with the same name.
        if self.executables.contains_key(&exe_name) {
            return Err(CellsError::ExecutableExists {
                cell_name: self.name.clone(),
                executable_name: exe_name,
            });
        }

        let _ = command.args(args);

        // Run 'pre_exec' hooks from the context of the soon-to-be launched child.
        let _ = {
            let exe_name_clone = exe_name.clone();
            unsafe {
                command
                    .pre_exec(move || aurae_process_pre_exec(&exe_name_clone))
            }
        };

        // Start the child process
        let exe = Executable::spawn(command)?;

        // Add the newly started child process to the cgroup
        let exe_pid = exe.pid();
        self.cgroup.add_task(exe.pid()).map_err(CellsError::from)?;

        info!(
            "Cells: cell_name={} executable_name={exe_name} spawn() -> pid={}",
            self.name, exe_pid.pid
        );

        // Ignoring return value as we've already assured ourselves that the key does not exist.
        let _ = self.executables.insert(exe_name, exe);

        Ok(())
    }

    pub fn kill_executable(
        &mut self,
        exe_name: &ExecutableName,
    ) -> Result<ExitStatus> {
        if let Some(exe) = self.executables.remove(exe_name) {
            Ok(exe.kill()?)
        } else {
            Err(CellsError::ExecutableNotFound {
                cell_name: self.name.clone(),
                executable_name: exe_name.clone(),
            })
        }
    }

    pub fn v2(&self) -> bool {
        self.cgroup.v2()
    }
}

fn aurae_process_pre_exec(exe_name: &ExecutableName) -> io::Result<()> {
    info!("CellService: aurae_process_pre_exec(): {exe_name}");
    // Here we are executing as the new spawned pid.
    // This is a place where we can "hook" into all processes
    // started with Aurae in the future. Similar to kprobe/uprobe
    // in Linux or LD_PRELOAD in libc.
    Ok(())
}

fn hierarchy() -> Box<dyn Hierarchy> {
    // Auraed will assume the V2 cgroup hierarchy by default.
    // For now we do not change this, albeit in theory we could
    // likely create backwards compatability for V1 hierarchy.
    //
    // For now, we simply... don't.
    // hierarchies::auto() // Uncomment to auto detect Cgroup hierarchy
    // hierarchies::V2
    Box::new(hierarchies::V2::new())
}
