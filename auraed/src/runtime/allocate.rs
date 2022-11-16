use crate::runtime::{AllocateCellRequest, AllocateCellResponse, Cell};
use tonic::{Response, Status};
use validation::{ValidatedType, ValidationError};
use validation_macros::{ValidatedType, ValidatingType};

#[derive(ValidatingType)]
pub(crate) struct ValidatedAllocateCellRequest {
    #[field_type(Option<Cell>)]
    cell: ValidatedCell,
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

impl ValidatedAllocateCellRequest {
    pub(crate) async fn execute(
        &self,
    ) -> Result<Response<AllocateCellResponse>, Status> {
        todo!()
    }
}

#[derive(ValidatedType)]
struct ValidatedCell {
    name: String,
    cpus: String,
    mems: String,
    shares: u64,
    quota: i64,
    ns_share_mount: bool,
    ns_share_uts: bool,
    ns_share_ipc: bool,
    ns_share_pid: bool,
    ns_share_net: bool,
    ns_share_cgroup: bool,
}

impl CellTypeValidator for CellValidator {
    fn validate_name(
        name: String,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<String, ValidationError> {
        let name = validation::required_not_empty(
            Some(name),
            field_name,
            parent_name,
        )?;

        validation::allow_regex(
            &name,
            &validation::DOMAIN_NAME_LABEL_REGEX,
            field_name,
            parent_name,
        )?;

        Ok(name)
    }

    fn validate_cpus(
        _cpus: String,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<String, ValidationError> {
        todo!()
    }

    fn validate_mems(
        _mems: String,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<String, ValidationError> {
        todo!()
    }

    fn validate_shares(
        _shares: u64,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<u64, ValidationError> {
        todo!()
    }

    fn validate_quota(
        _quota: i64,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<i64, ValidationError> {
        todo!()
    }

    fn validate_ns_share_mount(
        _ns_share_mount: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }

    fn validate_ns_share_uts(
        _ns_share_uts: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }

    fn validate_ns_share_ipc(
        _ns_share_ipc: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }

    fn validate_ns_share_pid(
        _ns_share_pid: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }

    fn validate_ns_share_net(
        _ns_share_net: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }

    fn validate_ns_share_cgroup(
        _ns_share_cgroup: bool,
        _field_name: &str,
        _parent_name: Option<&str>,
    ) -> Result<bool, ValidationError> {
        todo!()
    }
}
