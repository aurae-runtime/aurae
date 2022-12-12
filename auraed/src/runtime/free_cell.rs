use crate::runtime::cell_name::CellName;
use crate::runtime::CellService;
use aurae_proto::runtime::{FreeCellRequest, FreeCellResponse};
use tonic::Status;
use validation_macros::ValidatedType;

#[derive(ValidatedType)]
pub(crate) struct ValidatedFreeCellRequest {
    #[field_type(String)]
    #[validate]
    pub cell_name: CellName,
}

impl FreeCellRequestTypeValidator for FreeCellRequestValidator {}

impl ValidatedFreeCellRequest {
    pub(crate) fn handle(
        self,
        context: &CellService,
    ) -> Result<FreeCellResponse, Status> {
        let ValidatedFreeCellRequest { cell_name } = self;

        context.remove_cgroup(&cell_name).expect("remove cgroup");

        Ok(FreeCellResponse {})
    }
}
