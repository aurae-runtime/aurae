use std::{
    collections::HashMap,
    process::Child,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use cgroups_rs::Cgroup;

use super::{
    cell_name::CellName, error::Result, executable_name::ExecutableName,
};

#[derive(Debug, Default)]
struct Cell {
    cgroup: Cgroup,
    executables: HashMap<ExecutableName, Child>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct CellTable {
    cells: Arc<Mutex<HashMap<CellName, Cell>>>,
}

impl CellTable {
    pub(crate) fn add_cgroup(
        &self,
        cell_name: CellName,
        cgroup: Cgroup,
    ) -> Result<()> {
        let mut cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        // TODO: replace with this when it becomes stable
        // cell_table.try_insert(cell_name, cgroup)
        if cell_table.contains_key(&cell_name) {
            return Err(anyhow!("cgroup already exists for {cell_name}").into());
        }
        let _ = cell_table.insert(
            cell_name,
            Cell { cgroup, executables: Default::default() },
        );
        Ok(())
    }

    pub(crate) fn get_cgroup(
        &self,
        cell_name: &CellName,
    ) -> Result<Option<Cgroup>> {
        let cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        Ok(cell_table.get(cell_name).and_then(|cell| Some(cell.cgroup)))
    }

    pub(crate) fn rm_cgroup(&self, cell_name: &CellName) -> Result<Cgroup> {
        let mut cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        // TODO: check if executables is empty
        cell_table
            .remove(cell_name)
            .and_then(|cell| Some(cell.cgroup))
            .ok_or_else(|| anyhow!("failed to find {cell_name}").into())
    }

    fn add_child(
        &self,
        cell_name: CellName,
        exe_name: ExecutableName,
        child: Child,
    ) -> Result<()> {
        let cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        if let Some(cell) = cell_table.get(&cell_name) {
            let exes: &mut HashMap<ExecutableName, Child> =
                &mut cell.executables;
            // TODO: check for uniqueness of exe_name
            exes.insert(exe_name, child);
            return Ok(());
        }
        Err(anyhow!("failed to find Cell {cell_name}").into())
    }

    fn get_child(
        &self,
        cell_name: CellName,
        exe_name: ExecutableName,
    ) -> Result<Option<&Child>> {
        let cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        if let Some(cell) = cell_table.get(&cell_name).cloned() {
            return Ok(cell.executables.get(&exe_name));
        }
        Err(anyhow!("failed to find Cell {cell_name}").into())
    }

    fn rm_child(
        &self,
        cell_name: CellName,
        exe_name: ExecutableName,
    ) -> Result<Child> {
        let mut cell_table =
            self.cells.lock().map_err(|_| anyhow!("failed to lock table"))?;
        if let Some(cell) = cell_table.get(&cell_name) {
            return cell.executables.remove(&exe_name).ok_or_else(|| {
                anyhow!(
                    "failed to find Executable {exe_name} in Cell {cell_name}"
                )
                .into()
            });
        }
        Err(anyhow!("failed to find Cell {cell_name}").into())
    }
}
