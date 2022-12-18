use crate::runtime::cells::{CellName, ExecutableName};
use crate::runtime::{CpuCpus, CpuQuota, CpuWeight, CpusetMems};
use aurae_proto::runtime::{
    AllocateCellRequest, Cell, Executable, FreeCellRequest,
    StartExecutableRequest, StopExecutableRequest,
};
use validation::{ValidatedType, ValidationError};
use validation_macros::ValidatedType;

// TODO: Following the discord discussion of wanting to keep the logic on CellService,
//  versus on the validated request structs, we may not want to create a file per endpoint,
//  so I'm (future-highway) grouping it all here at least temporarily.

#[derive(ValidatedType)]
pub(crate) struct ValidatedAllocateCellRequest {
    #[field_type(Option<Cell>)]
    pub cell: ValidatedCell,
}

impl AllocateCellRequestTypeValidator for AllocateCellRequestValidator {
    fn validate_cell(
        cell: Option<Cell>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<ValidatedCell, ValidationError> {
        let cell = validation::required(cell, field_name, parent_name)?;

        ValidatedCell::validate(
            cell,
            Some(&validation::field_name(field_name, parent_name)),
        )
    }
}

#[derive(ValidatedType)]
pub(crate) struct ValidatedFreeCellRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
}

impl FreeCellRequestTypeValidator for FreeCellRequestValidator {}

#[derive(ValidatedType)]
pub(crate) struct ValidatedStartExecutableRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
    #[field_type(Option<Executable>)]
    pub executable: ValidatedExecutable,
}

impl StartExecutableRequestTypeValidator for StartExecutableRequestValidator {
    fn validate_executable(
        executable: Option<Executable>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<ValidatedExecutable, ValidationError> {
        let exe = validation::required(executable, field_name, parent_name)?;
        ValidatedExecutable::validate(exe, None) // TODO: parent name
    }
}

#[derive(ValidatedType)]
pub(crate) struct ValidatedStopExecutableRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
    #[field_type(String)]
    #[validate]
    pub executable_name: ExecutableName,
}

impl StopExecutableRequestTypeValidator for StopExecutableRequestValidator {}

// TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
#[derive(ValidatedType, Debug, Clone)]
pub(crate) struct ValidatedCell {
    #[field_type(String)]
    #[validate(create)]
    pub name: CellName,

    #[field_type(String)]
    #[validate(create)]
    pub cpu_cpus: CpuCpus,

    #[field_type(u64)]
    #[validate(create)]
    pub cpu_shares: CpuWeight,

    #[field_type(String)]
    #[validate(create)]
    pub cpu_mems: CpusetMems,

    #[field_type(i64)]
    #[validate(create)]
    pub cpu_quota: CpuQuota,
}

impl CellTypeValidator for CellValidator {}

impl From<ValidatedCell> for crate::runtime::cells::Cell {
    fn from(x: ValidatedCell) -> Self {
        Self::new(x)
    }
}

#[derive(ValidatedType, Debug)]
pub(crate) struct ValidatedExecutable {
    #[field_type(String)]
    #[validate(create)]
    pub name: ExecutableName,

    pub command: String,

    #[field_type(Vec<String>)]
    #[validate(none)]
    pub args: Vec<String>,

    // TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
    #[validate(none)]
    pub description: String,
}

impl ExecutableTypeValidator for ExecutableValidator {
    fn validate_command(
        command: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<String, ValidationError> {
        validation::required_not_empty(Some(command), field_name, parent_name)
    }
}

impl From<ValidatedExecutable> for crate::runtime::cells::Executable {
    fn from(x: ValidatedExecutable) -> Self {
        let ValidatedExecutable { name, command, args, description } = x;
        Self::new(name, command, args, description)
    }
}
