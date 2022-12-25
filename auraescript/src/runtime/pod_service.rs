use aurae_proto::runtime::*;

macros::ops_generator!(
    runtime,
    PodService,
    allocate(AllocatePodRequest) -> AllocatePodResponse,
    free(FreePodRequest) -> FreePodResponse,
    start(StartContainerRequest) -> StartContainerResponse,
    stop(StopContainerRequest) -> StopContainerResponse,
);
