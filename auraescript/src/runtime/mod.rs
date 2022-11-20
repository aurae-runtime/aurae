tonic::include_proto!("runtime");

macros::ops_generator!(
    runtime,
    CellService,
    allocate(AllocateCellRequest) -> AllocateCellResponse,
    free(FreeCellRequest) -> FreeCellResponse,
    start(StartCellRequest) -> StartCellResponse,
    stop(StopCellRequest) -> StopCellResponse,
);
