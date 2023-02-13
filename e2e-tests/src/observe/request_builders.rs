use std::cell;

use aurae_proto::{
    cells::{
        Cell, CellServiceAllocateRequest, CellServiceStartRequest, Executable,
    },
    observe::{GetPosixSignalsStreamRequest, Workload, WorkloadType},
};

struct CellBuilder {
    cell_name: Option<String>,
    parent: Option<String>,
    isolate_process: bool,
}

impl CellBuilder {
    pub fn new() -> Self {
        Self { cell_name: None, parent: None, isolate_process: false }
    }

    pub fn cell_name(&mut self, cell_name: String) -> &CellBuilder {
        self.cell_name = Some(cell_name);
        self
    }

    pub fn parent_cell_name(
        &mut self,
        parent_cell_name: String,
    ) -> &CellBuilder {
        self.parent = Some(parent_cell_name);
        self
    }

    pub fn isolate_process(&mut self) -> &CellBuilder {
        self.isolate_process = true;
        self
    }

    pub fn build(&self) -> Cell {
        let cell_name = if let Some(parent) = &self.parent {
            format!("{}/ae-e2e-{}", parent, uuid::Uuid::new_v4())
        } else {
            format!("ae-e2e-{}", uuid::Uuid::new_v4())
        };
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

pub(crate) struct CellServiceAllocateRequestBuilder {
    cell_builder: CellBuilder,
}

impl CellServiceAllocateRequestBuilder {
    pub fn new() -> Self {
        Self { cell_builder: CellBuilder::new() }
    }

    pub fn cell_name(
        &mut self,
        cell_name: String,
    ) -> &CellServiceAllocateRequestBuilder {
        self.cell_builder.cell_name(cell_name);
        self
    }

    pub fn parent_cell_name(
        &mut self,
        parent_cell_name: String,
    ) -> &CellServiceAllocateRequestBuilder {
        self.cell_builder.parent_cell_name(parent_cell_name);
        self
    }

    pub fn isolate_process(&mut self) -> &CellServiceAllocateRequestBuilder {
        self.cell_builder.isolate_process();
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
            name: String::from(format!("ae-sleeper-{}", uuid::Uuid::new_v4())),
            command: String::from(format!("sleep 400")),
            description: String::from("description"),
        }
    }

    pub fn executable_name(&mut self, name: String) -> &Self {
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
        self.executable_builder.executable_name(name);
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

pub(crate) struct GetPosixSignalsStreamRequestBuilder {
    workload: Option<Workload>,
}

impl GetPosixSignalsStreamRequestBuilder {
    pub fn new() -> Self {
        Self { workload: None }
    }

    pub fn cell_workload(
        &mut self,
        name: String,
    ) -> &GetPosixSignalsStreamRequestBuilder {
        self.workload = Some(Workload {
            workload_type: WorkloadType::Cell.into(),
            id: name,
        });
        self
    }

    pub fn build(&self) -> GetPosixSignalsStreamRequest {
        GetPosixSignalsStreamRequest { workload: self.workload.clone() }
    }
}
