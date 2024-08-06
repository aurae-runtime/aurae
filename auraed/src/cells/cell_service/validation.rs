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
use super::cells::{
    cgroups::{
        self,
        cpuset::{Cpus, Mems},
        CgroupSpec, Limit, Protection, Weight,
    },
    IsolationControls,
};
use super::executables::ExecutableName;
use crate::cells::cell_service::cells::CellName;
use proto::cells::{
    Cell, CellServiceAllocateRequest, CellServiceFreeRequest,
    CellServiceStartRequest, CellServiceStopRequest, CpuController,
    CpusetController, Executable, MemoryController,
};
use std::ffi::OsString;
use tokio::process::Command;
use validation::{ValidatedType, ValidationError};
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

#[derive(ValidatedType, Debug, Clone)]
pub struct ValidatedCell {
    #[field_type(String)]
    #[validate(create)]
    pub name: CellName,

    #[field_type(Option<CpuController>)]
    pub cpu: Option<ValidatedCpuController>,

    #[field_type(Option<CpusetController>)]
    pub cpuset: Option<ValidatedCpusetController>,

    #[field_type(Option<MemoryController>)]
    pub memory: Option<ValidatedMemoryController>,

    #[validate(none)]
    pub isolate_process: bool,

    #[validate(none)]
    pub isolate_network: bool,
}

impl CellTypeValidator for CellValidator {
    fn validate_cpu(
        cpu: Option<CpuController>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Option<ValidatedCpuController>, ValidationError> {
        let Some(cpu) = cpu else {
            return Ok(None);
        };

        Ok(Some(ValidatedCpuController::validate(
            cpu,
            Some(&*validation::field_name(field_name, parent_name)),
        )?))
    }

    fn validate_cpuset(
        cpuset: Option<CpusetController>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Option<ValidatedCpusetController>, ValidationError> {
        let Some(cpuset) = cpuset else {
            return Ok(None);
        };

        Ok(Some(ValidatedCpusetController::validate(
            cpuset,
            Some(&*validation::field_name(field_name, parent_name)),
        )?))
    }

    fn validate_memory(
        memory: Option<MemoryController>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Option<ValidatedMemoryController>, ValidationError> {
        let Some(memory) = memory else {
            return Ok(None);
        };

        Ok(Some(ValidatedMemoryController::validate(
            memory,
            Some(&*validation::field_name(field_name, parent_name)),
        )?))
    }
}

impl From<ValidatedCell> for super::cells::CellSpec {
    fn from(x: ValidatedCell) -> Self {
        let ValidatedCell {
            name: _,
            cpu,
            cpuset,
            memory,
            isolate_process,
            isolate_network,
        } = x;

        Self {
            cgroup_spec: CgroupSpec {
                cpu: cpu.map(|x| x.into()),
                cpuset: cpuset.map(|x| x.into()),
                memory: memory.map(|x| x.into()),
            },
            iso_ctl: IsolationControls { isolate_process, isolate_network },
        }
    }
}

#[derive(ValidatedType, Debug, Clone)]
pub struct ValidatedCpuController {
    #[field_type(Option<u64>)]
    #[validate(opt)]
    pub weight: Option<Weight>,

    #[field_type(Option<i64>)]
    #[validate(opt)]
    pub max: Option<Limit>,

    #[validate(none)]
    pub period: Option<u64>,
}

impl CpuControllerTypeValidator for CpuControllerValidator {}

impl From<ValidatedCpuController> for cgroups::cpu::CpuController {
    fn from(value: ValidatedCpuController) -> Self {
        let ValidatedCpuController { weight, max, period } = value;
        Self { weight, max, period }
    }
}

#[derive(ValidatedType, Debug, Clone)]
pub struct ValidatedCpusetController {
    #[field_type(Option<String>)]
    #[validate(opt)]
    pub cpus: Option<Cpus>,

    #[field_type(Option<String>)]
    #[validate(opt)]
    pub mems: Option<Mems>,
}

impl CpusetControllerTypeValidator for CpusetControllerValidator {}

impl From<ValidatedCpusetController> for cgroups::cpuset::CpusetController {
    fn from(value: ValidatedCpusetController) -> Self {
        let ValidatedCpusetController { cpus, mems } = value;
        Self { cpus, mems }
    }
}

#[derive(ValidatedType, Debug, Clone)]
pub struct ValidatedMemoryController {
    #[field_type(Option<i64>)]
    #[validate(opt)]
    pub min: Option<Protection>,

    #[field_type(Option<i64>)]
    #[validate(opt)]
    pub low: Option<Protection>,

    #[field_type(Option<i64>)]
    #[validate(opt)]
    pub high: Option<Limit>,

    #[field_type(Option<i64>)]
    #[validate(opt)]
    pub max: Option<Limit>,
}

impl MemoryControllerTypeValidator for MemoryControllerValidator {}

impl From<ValidatedMemoryController> for cgroups::memory::MemoryController {
    fn from(value: ValidatedMemoryController) -> Self {
        let ValidatedMemoryController { min, low, high, max } = value;
        Self { min, low, high, max }
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
    #[field_type(Option<String>)]
    #[validate(opt)]
    pub cell_name: Option<CellName>,
    #[field_type(Option<Executable>)]
    pub executable: ValidatedExecutable,
}

impl CellServiceStartRequestTypeValidator for CellServiceStartRequestValidator {
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
    #[field_type(Option<String>)]
    #[validate(opt)]
    pub cell_name: Option<CellName>,
    #[field_type(String)]
    #[validate]
    pub executable_name: ExecutableName,
}

impl CellServiceStopRequestTypeValidator for CellServiceStopRequestValidator {}

#[derive(ValidatedType, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_type_empty_cpu_valid() {
        let validated =
            CellValidator::validate_cpu(None, "field", Some("parent"));
        assert!(validated.is_ok());
        assert!(validated.unwrap().is_none());
    }

    #[test]
    fn test_cell_type_cpu_valid() {
        let validated = CellValidator::validate_cpu(
            Some(CpuController { weight: Some(1000), max: None, period: None }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_ok());
        let inner = validated.unwrap();
        assert!(inner.is_some());
        let controller = inner.unwrap();
        assert_eq!(controller.weight, Some(Weight::new(1000)));
        assert_eq!(controller.max, None);
        assert_eq!(controller.period, None);
    }

    #[test]
    fn test_cell_type_cpu_weight_too_small() {
        let validated = CellValidator::validate_cpu(
            Some(CpuController { weight: Some(0), max: None, period: None }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_cpu_weight_too_large() {
        let validated = CellValidator::validate_cpu(
            Some(CpuController {
                weight: Some(10001),
                max: None,
                period: None,
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_cpu_max_too_small() {
        let validated = CellValidator::validate_cpu(
            Some(CpuController {
                weight: Some(1000),
                max: Some(-1),
                period: None,
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_cpuset_valid() {
        let validated = CellValidator::validate_cpuset(
            Some(CpusetController {
                cpus: Some(String::from("1,2-4")),
                mems: Some(String::from("1-4")),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_ok());
        let inner = validated.unwrap();
        assert!(inner.is_some());
        let controller = inner.unwrap();
        assert_eq!(controller.cpus, Some(Cpus::new(String::from("1,2-4"))));
        assert_eq!(controller.mems, Some(Mems::new(String::from("1-4"))));
    }

    #[test]
    fn test_cell_type_cpuset_invalid_cpus() {
        let validated = CellValidator::validate_cpuset(
            Some(CpusetController {
                cpus: Some(String::from("foo")),
                mems: Some(String::from("1-4")),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_cpuset_invalid_mems() {
        let validated = CellValidator::validate_cpuset(
            Some(CpusetController {
                cpus: Some(String::from("1,2-4")),
                mems: Some(String::from("1..4")),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_memory_valid() {
        let validated = CellValidator::validate_memory(
            Some(MemoryController {
                min: None,
                low: Some(1000),
                high: None,
                max: Some(10000),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_ok());
        let inner = validated.unwrap();
        assert!(inner.is_some());
        let controller = inner.unwrap();
        assert_eq!(controller.min, None);
        assert_eq!(controller.low, Some(Protection::new(1000)));
        assert_eq!(controller.high, None);
        assert_eq!(controller.max, Some(Limit::new(10000)));
    }

    #[test]
    fn test_cell_type_memory_low_too_small() {
        let validated = CellValidator::validate_memory(
            Some(MemoryController {
                min: None,
                low: Some(-1),
                high: None,
                max: Some(10000),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_type_memory_max_too_small() {
        let validated = CellValidator::validate_memory(
            Some(MemoryController {
                min: None,
                low: Some(1000),
                high: None,
                max: Some(-1),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_service_start_request_empty_executable() {
        let validated = CellServiceStartRequestValidator::validate_executable(
            None,
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_service_start_request_empty_command() {
        let validated = CellServiceStartRequestValidator::validate_executable(
            Some(Executable {
                command: String::from(""),
                name: String::from("name"),
                description: String::from("description"),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_err());
    }

    #[test]
    fn test_cell_service_start_request_valid() {
        let validated = CellServiceStartRequestValidator::validate_executable(
            Some(Executable {
                command: String::from("command"),
                name: String::from("name"),
                description: String::from("description"),
            }),
            "field",
            Some("parent"),
        );
        assert!(validated.is_ok());
        assert_eq!(
            validated.unwrap(),
            ValidatedExecutable {
                name: ExecutableName::new(String::from("name")),
                description: String::from("description"),
                command: OsString::from("command"),
            },
        );
    }

    #[test]
    fn test_executable_empty_command() {
        assert!(ExecutableValidator::validate_command(
            String::from(""),
            "field",
            Some("parent")
        )
        .is_err());
    }

    #[test]
    fn test_executable_valid() {
        let validated = ExecutableValidator::validate_command(
            String::from("command"),
            "field",
            Some("parent"),
        );
        assert!(validated.is_ok());
        assert_eq!(validated.unwrap(), OsString::from("command"));
    }
}