use aurae_proto::runtime::*;

macros::ops_generator!(
    runtime,
    CellService,
    allocate(AllocateCellRequest) -> AllocateCellResponse,
    free(FreeCellRequest) -> FreeCellResponse,
    start(StartExecutableRequest) -> StartExecutableResponse,
    stop(StopExecutableRequest) -> StopExecutableResponse,
);
