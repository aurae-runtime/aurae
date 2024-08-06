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
#![allow(unused)]

use proto::cells::{
    Cell, CellServiceAllocateRequest, CellServiceStartRequest, Executable,
};

fn generate_cell_name(parent_name: Option<&str>) -> String {
    if let Some(parent_name) = parent_name {
        format!("{parent_name}/ae-e2e-{}", uuid::Uuid::new_v4())
    } else {
        format!("ae-e2e-{}", uuid::Uuid::new_v4())
    }
}

struct CellBuilder {
    parent: Option<String>,
    isolate_process: bool,
}

impl CellBuilder {
    pub fn new() -> Self {
        Self { parent: None, isolate_process: false }
    }

    pub fn parent_cell_name(&mut self, parent_cell_name: String) -> &mut Self {
        self.parent = Some(parent_cell_name);
        self
    }

    pub fn isolate_process(&mut self) -> &mut Self {
        self.isolate_process = true;
        self
    }

    pub fn build(&self) -> Cell {
        let cell_name = generate_cell_name(self.parent.as_deref());
        Cell {
            name: cell_name,
            cpu: None,
            cpuset: None,
            memory: None,
            isolate_network: false,
            isolate_process: self.isolate_process,
        }
    }
}

pub struct CellServiceAllocateRequestBuilder {
    cell_builder: CellBuilder,
}

impl CellServiceAllocateRequestBuilder {
    pub fn new() -> Self {
        Self { cell_builder: CellBuilder::new() }
    }

    pub fn parent_cell_name(&mut self, parent_cell_name: String) -> &mut Self {
        let _ = self.cell_builder.parent_cell_name(parent_cell_name);
        self
    }

    pub fn isolate_process(&mut self) -> &mut Self {
        let _ = self.cell_builder.isolate_process();
        self
    }

    pub fn build(&self) -> CellServiceAllocateRequest {
        CellServiceAllocateRequest { cell: Some(self.cell_builder.build()) }
    }
}

struct ExecutableBuilder {
    name: String,
    command: String,
    description: String,
}

impl ExecutableBuilder {
    pub fn new() -> Self {
        Self {
            name: format!("ae-sleeper-{}", uuid::Uuid::new_v4()),
            command: "sleep 400".to_string(),
            description: String::from("description"),
        }
    }

    pub fn executable_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn build(&self) -> Executable {
        Executable {
            name: self.name.clone(),
            command: self.command.clone(),
            description: self.description.clone(),
        }
    }
}

pub(crate) struct CellServiceStartRequestBuilder {
    cell_name: Option<String>,
    executable_builder: ExecutableBuilder,
}

impl CellServiceStartRequestBuilder {
    pub fn new() -> Self {
        Self { cell_name: None, executable_builder: ExecutableBuilder::new() }
    }

    pub fn cell_name(&mut self, cell_name: String) -> &mut Self {
        self.cell_name = Some(cell_name);
        self
    }

    pub fn executable_name(&mut self, name: String) -> &mut Self {
        let _ = self.executable_builder.executable_name(name);
        self
    }

    pub fn build(&self) -> CellServiceStartRequest {
        assert!(self.cell_name.is_some(), "cell_name needs to be set");
        CellServiceStartRequest {
            cell_name: self.cell_name.clone(),
            executable: Some(self.executable_builder.build()),
        }
    }
}