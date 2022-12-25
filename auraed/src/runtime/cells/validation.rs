use aurae_cells::{
    CellName, CgroupSpec, CpuCpus, CpuQuota, CpuWeight, CpusetMems,
};
use aurae_executables::{ExecutableName, SharedNamespaces};
use aurae_proto::runtime::{
    AllocateCellRequest, Cell, Executable, FreeCellRequest,
    StartExecutableRequest, StopExecutableRequest,
};
use std::ffi::CString;
use validation::{ValidatedType, ValidationError};
use validation_macros::ValidatedType;

// TODO: Following the discord discussion of wanting to keep the logic on CellService,
//  versus on the validated request structs, we may not want to create a file per endpoint,
//  so I'm (future-highway) grouping it all here at least temporarily.
// TODO: ...and I (@krisnova) read the above statement.

#[derive(Debug, ValidatedType)]
pub struct ValidatedAllocateCellRequest {
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

#[derive(Debug, ValidatedType)]
pub struct ValidatedFreeCellRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
}

impl FreeCellRequestTypeValidator for FreeCellRequestValidator {}

#[derive(Debug, ValidatedType)]
pub struct ValidatedStartExecutableRequest {
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

#[derive(Debug, ValidatedType)]
pub struct ValidatedStopExecutableRequest {
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
    pub ns_share_uts: bool,
    #[validate(none)]
    pub ns_share_ipc: bool,
    #[validate(none)]
    pub ns_share_pid: bool,
    #[validate(none)]
    pub ns_share_net: bool,
    #[validate(none)]
    pub ns_share_cgroup: bool,
}

impl CellTypeValidator for CellValidator {}

impl From<ValidatedCell> for aurae_cells::CellSpec {
    fn from(x: ValidatedCell) -> Self {
        let ValidatedCell {
            name: _,
            cpu_cpus,
            cpu_shares,
            cpu_mems,
            cpu_quota,
            ns_share_mount,
            ns_share_uts,
            ns_share_ipc,
            ns_share_pid,
            ns_share_net,
            ns_share_cgroup,
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
                uts: ns_share_uts,
                ipc: ns_share_ipc,
                pid: ns_share_pid,
                net: ns_share_net,
                cgroup: ns_share_cgroup,
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
    pub command: CString,

    // TODO: `#[validate(none)] is used to skip validation. Actually validate when restrictions are known.
    #[validate(none)]
    pub description: String,
}

impl ExecutableTypeValidator for ExecutableValidator {
    fn validate_command(
        command: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<CString, ValidationError> {
        let command = validation::required_not_empty(
            Some(command),
            field_name,
            parent_name,
        )?;

        let command =
            CString::new(command).map_err(|_| ValidationError::Invalid {
                field: validation::field_name(field_name, parent_name),
            })?;

        Ok(command)
    }
}

impl From<ValidatedExecutable> for aurae_executables::ExecutableSpec {
    fn from(x: ValidatedExecutable) -> Self {
        let ValidatedExecutable { name, command, description } = x;
        Self { name, command, description }
    }
}
