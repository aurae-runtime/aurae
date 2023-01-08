use super::{
    cells::{CellName, CgroupSpec, CpuCpus, CpuQuota, CpuWeight, CpusetMems},
    executables::{auraed::SharedNamespaces, ExecutableName},
};
use aurae_proto::runtime::{
    Cell, CellServiceAllocateRequest, CellServiceFreeRequest,
    CellServiceStartRequest, CellServiceStopRequest, Executable,
};
use std::collections::VecDeque;
use std::ffi::OsString;
use tokio::process::Command;
use validation::{ValidatedField, ValidatedType, ValidationError};
use validation_macros::ValidatedType;

// TODO: Following the discord discussion of wanting to keep the logic on CellService,
//  versus on the validated request structs, we may not want to create a file per endpoint,
//  so I'm (future-highway) grouping it all here at least temporarily.
// TODO: ...and I (@krisnova) read the above statement.

#[derive(Debug, ValidatedType)]
pub struct ValidatedCellServiceAllocateRequest {
    #[field_type(Option<Cell>)]
    pub cell: ValidatedCell,
}

impl CellServiceAllocateRequestTypeValidator
    for CellServiceAllocateRequestValidator
{
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

#[derive(Debug, ValidatedType)]
pub struct ValidatedCellServiceFreeRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
}

impl CellServiceFreeRequestTypeValidator for CellServiceFreeRequestValidator {}

#[derive(Debug, ValidatedType)]
pub struct ValidatedCellServiceStartRequest {
    #[field_type(String)]
    pub cell_name: VecDeque<CellName>,
    #[field_type(Option<Executable>)]
    pub executable: ValidatedExecutable,
}

impl CellServiceStartRequestTypeValidator for CellServiceStartRequestValidator {
    fn validate_cell_name(
        cell_name: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<VecDeque<CellName>, ValidationError> {
        let cell_name =
            validation::required(Some(cell_name), field_name, parent_name)?;

        let cell_name = cell_name
            .split('/')
            .flat_map(|cell_name| {
                CellName::validate_for_creation(
                    Some(cell_name.into()),
                    field_name,
                    parent_name,
                )
            })
            .collect();

        Ok(cell_name)
    }

    fn validate_executable(
        executable: Option<Executable>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<ValidatedExecutable, ValidationError> {
        let executable =
            validation::required(executable, field_name, parent_name)?;
        ValidatedExecutable::validate(
            executable,
            Some(&*validation::field_name(field_name, parent_name)),
        )
    }
}

#[derive(Debug, ValidatedType)]
pub struct ValidatedCellServiceStopRequest {
    #[field_type(String)]
    pub cell_name: VecDeque<CellName>,
    #[field_type(String)]
    #[validate]
    pub executable_name: ExecutableName,
}

impl CellServiceStopRequestTypeValidator for CellServiceStopRequestValidator {
    fn validate_cell_name(
        cell_name: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<VecDeque<CellName>, ValidationError> {
        // TODO: refactor to a CellNamePath maybe
        let cell_name =
            validation::required(Some(cell_name), field_name, parent_name)?;

        let cell_name = cell_name
            .split('/')
            .flat_map(|cell_name| {
                CellName::validate_for_creation(
                    Some(cell_name.into()),
                    field_name,
                    parent_name,
                )
            })
            .collect();

        Ok(cell_name)
    }
}

#[derive(ValidatedType, Debug, Clone)]
pub struct ValidatedCell {
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

    #[validate(none)]
    pub ns_share_mount: bool,
    #[validate(none)]
    pub ns_share_pid: bool,
    #[validate(none)]
    pub ns_share_net: bool,
}

impl CellTypeValidator for CellValidator {}

impl From<ValidatedCell> for super::cells::CellSpec {
    fn from(x: ValidatedCell) -> Self {
        let ValidatedCell {
            name: _,
            cpu_cpus,
            cpu_shares,
            cpu_mems,
            cpu_quota,
            ns_share_mount,
            ns_share_pid,
            ns_share_net,
        } = x;

        Self {
            cgroup_spec: CgroupSpec {
                cpu_cpus,
                cpu_quota,
                cpu_weight: cpu_shares,
                cpuset_mems: cpu_mems,
            },
            shared_namespaces: SharedNamespaces {
                mount: ns_share_mount,
                pid: ns_share_pid,
                net: ns_share_net,
            },
        }
    }
}

#[derive(ValidatedType, Debug)]
pub struct ValidatedExecutable {
    #[field_type(String)]
    #[validate(create)]
    pub name: ExecutableName,

    #[field_type(String)]
    pub command: OsString,

    // TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
    #[validate(none)]
    pub description: String,
}

impl ExecutableTypeValidator for ExecutableValidator {
    fn validate_command(
        command: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<OsString, ValidationError> {
        let command = validation::required_not_empty(
            Some(command),
            field_name,
            parent_name,
        )?;

        Ok(OsString::from(command))
    }
}

impl From<ValidatedExecutable> for super::executables::ExecutableSpec {
    fn from(x: ValidatedExecutable) -> Self {
        let ValidatedExecutable { name, command, description } = x;

        let mut c = Command::new("sh");
        let _ = c.args([OsString::from("-c"), command]);

        // We are checking that command has an arg to assure ourselves that `command.arg`
        // mutates command, and is not making a clone to return
        assert_eq!(c.as_std().get_args().len(), 2);

        Self { name, command: c, description }
    }
}
