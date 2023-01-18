/* -------------------------------------------------------------------------- *\
#             Apache 2.0 License Copyright © The Aurae Authors                #
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

// TODO: macro doesn't support streaming. Does deno?
macros::ops_generator!(
    kubernetes::cri,
    {
        RuntimeService,
        version(VersionRequest) -> VersionResponse,
        run_pod_sandbox(RunPodSandboxRequest) -> RunPodSandboxResponse,
        stop_pod_sandbox(StopPodSandboxRequest) -> StopPodSandboxResponse,
        remove_pod_sandbox(RemovePodSandboxRequest) -> RemovePodSandboxResponse,
        pod_sandbox_status(PodSandboxStatusRequest) -> PodSandboxStatusResponse,
        list_pod_sandbox(ListPodSandboxRequest) -> ListPodSandboxResponse,
        create_container(CreateContainerRequest) -> CreateContainerResponse,
        start_container(StartContainerRequest) -> StartContainerResponse,
        stop_container(StopContainerRequest) -> StopContainerResponse,
        remove_container(RemoveContainerRequest) -> RemoveContainerResponse,
        list_containers(ListContainersRequest) -> ListContainersResponse,
        container_status(ContainerStatusRequest) -> ContainerStatusResponse,
        update_container_resources(UpdateContainerResourcesRequest) -> UpdateContainerResourcesResponse,
        reopen_container_log(ReopenContainerLogRequest) -> ReopenContainerLogResponse,
        exec_sync(ExecSyncRequest) -> ExecSyncResponse,
        exec(ExecRequest) -> ExecResponse,
        attach(AttachRequest) -> AttachResponse,
        port_forward(PortForwardRequest) -> PortForwardResponse,
        container_stats(ContainerStatsRequest) -> ContainerStatsResponse,
        list_container_stats(ListContainerStatsRequest) -> ListContainerStatsResponse,
        pod_sandbox_stats(PodSandboxStatsRequest) -> PodSandboxStatsResponse,
        list_pod_sandbox_stats(ListPodSandboxStatsRequest) -> ListPodSandboxStatsResponse,
        update_runtime_config(UpdateRuntimeConfigRequest) -> UpdateRuntimeConfigResponse,
        status(StatusRequest) -> StatusResponse,
        checkpoint_container(CheckpointContainerRequest) -> CheckpointContainerResponse,
        // get_container_events(GetEventsRequest) -> [ContainerEventResponse],
        list_metric_descriptors(ListMetricDescriptorsRequest) -> ListMetricDescriptorsResponse,
        list_pod_sandbox_metrics(ListPodSandboxMetricsRequest) -> ListPodSandboxMetricsResponse,
    },
    {
        ImageService,
        list_images(ListImagesRequest) -> ListImagesResponse,
        image_status(ImageStatusRequest) -> ImageStatusResponse,
        pull_image(PullImageRequest) -> PullImageResponse,
        remove_image(RemoveImageRequest) -> RemoveImageResponse,
        image_fs_info(ImageFsInfoRequest) -> ImageFsInfoResponse,
    }
);
