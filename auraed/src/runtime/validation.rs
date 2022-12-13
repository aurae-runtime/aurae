use crate::runtime::cell_name::CellName;
use crate::runtime::executable_name::ExecutableName;
use aurae_proto::runtime::{
    AllocateCellRequest, Cell, Executable, FreeCellRequest, StartCellRequest,
    StopCellRequest,
};
use std::process::Command;
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
pub(crate) struct ValidatedStartCellRequest {
    #[field_type(Option<Executable>)]
    pub executable: ValidatedExecutable,
}

impl StartCellRequestTypeValidator for StartCellRequestValidator {
    fn validate_executable(
        executable: Option<Executable>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<ValidatedExecutable, ValidationError> {
        let executable =
            validation::required(executable, field_name, parent_name)?;

        ValidatedExecutable::validate(
            executable,
            Some(&validation::field_name(field_name, parent_name)),
        )
    }
}

#[derive(ValidatedType)]
pub(crate) struct ValidatedStopCellRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
    #[field_type(String)]
    #[validate]
    pub executable_name: ExecutableName,
}

impl StopCellRequestTypeValidator for StopCellRequestValidator {}

// TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
#[derive(ValidatedType)]
pub(crate) struct ValidatedCell {
    #[field_type(String)]
    pub name: CellName,
    #[validate(none)]
    pub cpu_cpus: String,
    #[validate(none)]
    pub cpu_shares: u64,
    #[validate(none)]
    pub cpu_mems: String,
    #[validate(none)]
    pub cpu_quota: i64,
}

impl CellTypeValidator for CellValidator {
    fn validate_name(
        name: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<CellName, ValidationError> {
        CellName::validate_for_creation(Some(name), field_name, parent_name)
    }
}

#[derive(ValidatedType)]
pub(crate) struct ValidatedExecutable {
    #[field_type(String)]
    pub name: ExecutableName,
    #[field_type(String)]
    pub command: Command,
    // TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
    #[validate(none)]
    pub description: String,
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
}

impl ExecutableTypeValidator for ExecutableValidator {
    fn validate_name(
        name: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<ExecutableName, ValidationError> {
        ExecutableName::validate_for_creation(
            Some(name),
            field_name,
            parent_name,
        )
    }

    fn validate_command(
        command: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Command, ValidationError> {
        let command = validation::required_not_empty(
            Some(command),
            field_name,
            parent_name,
        )?;

        let command = super::command_from_string(&command).map_err(|_| {
            ValidationError::Invalid {
                field: validation::field_name(field_name, parent_name),
            }
        })?;

        Ok(command)
    }
}
