# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [discovery.proto](#discovery-proto)
    - [DiscoverRequest](#aurae-discovery-v0-DiscoverRequest)
    - [DiscoverResponse](#aurae-discovery-v0-DiscoverResponse)
  
    - [DiscoveryService](#aurae-discovery-v0-DiscoveryService)
  
- [observe.proto](#observe-proto)
    - [GetAuraeDaemonLogStreamRequest](#aurae-observe-v0-GetAuraeDaemonLogStreamRequest)
    - [GetSubProcessStreamRequest](#aurae-observe-v0-GetSubProcessStreamRequest)
    - [LogItem](#aurae-observe-v0-LogItem)
  
    - [LogChannelType](#aurae-observe-v0-LogChannelType)
  
    - [ObserveService](#aurae-observe-v0-ObserveService)
  
- [runtime.proto](#runtime-proto)
    - [Cell](#aurae-runtime-v0-Cell)
    - [CellServiceAllocateRequest](#aurae-runtime-v0-CellServiceAllocateRequest)
    - [CellServiceAllocateResponse](#aurae-runtime-v0-CellServiceAllocateResponse)
    - [CellServiceFreeRequest](#aurae-runtime-v0-CellServiceFreeRequest)
    - [CellServiceFreeResponse](#aurae-runtime-v0-CellServiceFreeResponse)
    - [CellServiceStartRequest](#aurae-runtime-v0-CellServiceStartRequest)
    - [CellServiceStartResponse](#aurae-runtime-v0-CellServiceStartResponse)
    - [CellServiceStopRequest](#aurae-runtime-v0-CellServiceStopRequest)
    - [CellServiceStopResponse](#aurae-runtime-v0-CellServiceStopResponse)
    - [Container](#aurae-runtime-v0-Container)
    - [CpuController](#aurae-runtime-v0-CpuController)
    - [CpusetController](#aurae-runtime-v0-CpusetController)
    - [Executable](#aurae-runtime-v0-Executable)
    - [Pod](#aurae-runtime-v0-Pod)
    - [PodServiceAllocateRequest](#aurae-runtime-v0-PodServiceAllocateRequest)
    - [PodServiceAllocateResponse](#aurae-runtime-v0-PodServiceAllocateResponse)
    - [PodServiceFreeRequest](#aurae-runtime-v0-PodServiceFreeRequest)
    - [PodServiceFreeResponse](#aurae-runtime-v0-PodServiceFreeResponse)
    - [PodServiceStartRequest](#aurae-runtime-v0-PodServiceStartRequest)
    - [PodServiceStartResponse](#aurae-runtime-v0-PodServiceStartResponse)
    - [PodServiceStopRequest](#aurae-runtime-v0-PodServiceStopRequest)
    - [PodServiceStopResponse](#aurae-runtime-v0-PodServiceStopResponse)
  
    - [CellService](#aurae-runtime-v0-CellService)
    - [InstanceService](#aurae-runtime-v0-InstanceService)
    - [PodService](#aurae-runtime-v0-PodService)
    - [SpawnService](#aurae-runtime-v0-SpawnService)
  
- [Scalar Value Types](#scalar-value-types)



<a name="discovery-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## discovery.proto



<a name="aurae-discovery-v0-DiscoverRequest"></a>

### DiscoverRequest







<a name="aurae-discovery-v0-DiscoverResponse"></a>

### DiscoverResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| healthy | [bool](#bool) |  |  |
| version | [string](#string) |  |  |





 

 

 


<a name="aurae-discovery-v0-DiscoveryService"></a>

### DiscoveryService


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Discover | [DiscoverRequest](#aurae-discovery-v0-DiscoverRequest) | [DiscoverResponse](#aurae-discovery-v0-DiscoverResponse) | Used to confirm that the host is running Aurae and to get some / information including the version of Aurae that is running. |

 



<a name="observe-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## observe.proto



<a name="aurae-observe-v0-GetAuraeDaemonLogStreamRequest"></a>

### GetAuraeDaemonLogStreamRequest







<a name="aurae-observe-v0-GetSubProcessStreamRequest"></a>

### GetSubProcessStreamRequest
TODO: not implemented


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel_type | [LogChannelType](#aurae-observe-v0-LogChannelType) |  |  |
| process_id | [int64](#int64) |  |  |






<a name="aurae-observe-v0-LogItem"></a>

### LogItem



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel | [string](#string) |  |  |
| line | [string](#string) |  |  |
| timestamp | [int64](#int64) |  |  |





 


<a name="aurae-observe-v0-LogChannelType"></a>

### LogChannelType


| Name | Number | Description |
| ---- | ------ | ----------- |
| LOG_CHANNEL_TYPE_UNSPECIFIED | 0 |  |
| LOG_CHANNEL_TYPE_STDOUT | 1 |  |
| LOG_CHANNEL_TYPE_STDERR | 2 |  |


 

 


<a name="aurae-observe-v0-ObserveService"></a>

### ObserveService


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| GetAuraeDaemonLogStream | [GetAuraeDaemonLogStreamRequest](#aurae-observe-v0-GetAuraeDaemonLogStreamRequest) | [LogItem](#aurae-observe-v0-LogItem) stream | request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). |
| GetSubProcessStream | [GetSubProcessStreamRequest](#aurae-observe-v0-GetSubProcessStreamRequest) | [LogItem](#aurae-observe-v0-LogItem) stream | TODO: request log stream for a sub process |

 



<a name="runtime-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## runtime.proto



<a name="aurae-runtime-v0-Cell"></a>

### Cell
An isolation resource used to divide a system into smaller resource
/ boundaries.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | Resource parameters for control groups (cgroups) / Build on the [cgroups-rs](https://github.com/kata-containers/cgroups-rs) / crate. See / [examples](https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs) |
| cpu | [CpuController](#aurae-runtime-v0-CpuController) |  |  |
| cpuset | [CpusetController](#aurae-runtime-v0-CpusetController) |  |  |
| isolate_process | [bool](#bool) |  | Will isolate the process (and proc filesystem) from the host. / Will unshare the pid, ipc, uts, and mount namespaces. / The cgroup namespace is always unshared with the host. / / Default: false |
| isolate_network | [bool](#bool) |  | Will isolate the network from the host. / Will unshare the net namespaces. / The cgroup namespace is always unshared with the host. / / Default: false |






<a name="aurae-runtime-v0-CellServiceAllocateRequest"></a>

### CellServiceAllocateRequest
An Aurae cell is a name given to Linux control groups (cgroups) that also include
/ a name, and special pre-exec functionality that is executed from within the same context
/ as any executables scheduled.
/
/ A cell must be allocated for every executable scheduled. A cell defines the resource
/ constraints of the system to allocate for an arbitrary use case.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell | [Cell](#aurae-runtime-v0-Cell) |  | A smaller resource constrained section of the system. |






<a name="aurae-runtime-v0-CellServiceAllocateResponse"></a>

### CellServiceAllocateResponse
The response after a cell has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| cgroup_v2 | [bool](#bool) |  | A bool that will be set to true if the cgroup was created with / cgroup v2 controller. |






<a name="aurae-runtime-v0-CellServiceFreeRequest"></a>

### CellServiceFreeRequest
Used to remove or free a cell after it has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |






<a name="aurae-runtime-v0-CellServiceFreeResponse"></a>

### CellServiceFreeResponse
Response after removing or freeing a cell.






<a name="aurae-runtime-v0-CellServiceStartRequest"></a>

### CellServiceStartRequest
A request for starting an executable inside of a Cell.
/
/ This is the lowest level of raw executive functionality.
/ Here you can define shell commands, and meta information about the command.
/ An executable is started synchronously.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable | [Executable](#aurae-runtime-v0-Executable) |  |  |






<a name="aurae-runtime-v0-CellServiceStartResponse"></a>

### CellServiceStartResponse
The response after starting an executable within a Cell.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pid | [int32](#int32) |  | Return a pid as an int32 based on the pid_t type / in various libc libraries. |






<a name="aurae-runtime-v0-CellServiceStopRequest"></a>

### CellServiceStopRequest
Request to stop an executable at runtime.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable_name | [string](#string) |  |  |






<a name="aurae-runtime-v0-CellServiceStopResponse"></a>

### CellServiceStopResponse







<a name="aurae-runtime-v0-Container"></a>

### Container



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | The name of the container. |
| image | [string](#string) |  | Define a remote container image. / / This should be a fully qualified URI and not a container &#34;shortname&#34;. / The file type that is returned should be an uncompresed OCI compatible container &#34;bundle&#34; / as defined in the [OCI spec](https://github.com/opencontainers/runtime-spec/blob/main/bundle.md#filesystem-bundle) / / ## Building a container bundle from an existing OCI image / / OCI &#34;images&#34; are effectively just tarballs. You can assemble / a bundle from an existing known image. / / ```bash / cd examples / mkdir -p aurae-busybox/rootfs / docker pull busybox / docker create --name aurae-busybox busybox / docker export aurae-busybox | tar -xfC aurae-busybox/rootfs - / cd aurae-busybox / runc spec / ``` / / Aurae will default pull down am image from a remote location and save to the Aurae socket directory as follows. / / ``` / $AURAE_SOCK_PATH/bundle/$NAME /``` / |
| registry | [string](#string) |  | Define a public portion of a container registry. / / Such as: / - ghcr.io / - https://registry.hub.docker.com / / Registry strings will be joined at runtime with the image / string such that a working container bundle path can be formed. |






<a name="aurae-runtime-v0-CpuController"></a>

### CpuController
Docs: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpu


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| weight | [uint64](#uint64) | optional | Weight of how much of the total CPU time should this control group get. Note that this is hierarchical, so this is weighted against the siblings of this control group.

* Minimum: 1 * Maximum: 10_000 |
| max | [int64](#int64) | optional | In one period (1_000_000), how much can the tasks run.

* Minimum: 0

By default a cgroup has no limit, represented as the literal string &#34;max&#34;. Not settings this field retains the default of no limit. |






<a name="aurae-runtime-v0-CpusetController"></a>

### CpusetController
Docs: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpuset


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cpus | [string](#string) | optional | A comma-separated list of CPU IDs where the task in the control group can run. Dashes between numbers indicate ranges. |
| mems | [string](#string) | optional | Same syntax as the cpus field of this structure, but applies to memory nodes instead of processors. |






<a name="aurae-runtime-v0-Executable"></a>

### Executable
The most primitive workload in Aurae, a standard executable process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |
| command | [string](#string) |  |  |
| description | [string](#string) |  |  |






<a name="aurae-runtime-v0-Pod"></a>

### Pod
OCI image represents a filesystem bundle on disk using familiar parlance.
/
/ OCI Filesystem Bundle: https://github.com/opencontainers/runtime-spec/blob/main/bundle.md#filesystem-bundle


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | Name is the name of the pod. |






<a name="aurae-runtime-v0-PodServiceAllocateRequest"></a>

### PodServiceAllocateRequest
The request to allocate a Pod.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pod | [Pod](#aurae-runtime-v0-Pod) |  | A boring set of containers with shared network and disk. |






<a name="aurae-runtime-v0-PodServiceAllocateResponse"></a>

### PodServiceAllocateResponse







<a name="aurae-runtime-v0-PodServiceFreeRequest"></a>

### PodServiceFreeRequest







<a name="aurae-runtime-v0-PodServiceFreeResponse"></a>

### PodServiceFreeResponse







<a name="aurae-runtime-v0-PodServiceStartRequest"></a>

### PodServiceStartRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |






<a name="aurae-runtime-v0-PodServiceStartResponse"></a>

### PodServiceStartResponse







<a name="aurae-runtime-v0-PodServiceStopRequest"></a>

### PodServiceStopRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pod_name | [string](#string) |  |  |
| container_name | [string](#string) |  |  |






<a name="aurae-runtime-v0-PodServiceStopResponse"></a>

### PodServiceStopResponse






 

 

 


<a name="aurae-runtime-v0-CellService"></a>

### CellService
Cells is the most fundamental isolation boundary for Aurae.
/ A cell is an isolate set of resources of the system which can be
/ used to run workloads.
/
/ A cell is composed of a unique cgroup namespace, and unshared kernel
/ namespaces.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [CellServiceAllocateRequest](#aurae-runtime-v0-CellServiceAllocateRequest) | [CellServiceAllocateResponse](#aurae-runtime-v0-CellServiceAllocateResponse) | Reserve requested system resources for a new cell. / For cells specifically this will allocate and reserve cgroup resources / only. |
| Free | [CellServiceFreeRequest](#aurae-runtime-v0-CellServiceFreeRequest) | [CellServiceFreeResponse](#aurae-runtime-v0-CellServiceFreeResponse) | Free up previously requested resources for an existing cell |
| Start | [CellServiceStartRequest](#aurae-runtime-v0-CellServiceStartRequest) | [CellServiceStartResponse](#aurae-runtime-v0-CellServiceStartResponse) | Start a new Executable inside of an existing cell. Can be called / in serial to start more than one executable in the same cell. |
| Stop | [CellServiceStopRequest](#aurae-runtime-v0-CellServiceStopRequest) | [CellServiceStopResponse](#aurae-runtime-v0-CellServiceStopResponse) | Stop one or more Executables inside of an existing cell. / Can be called in serial to stop/retry more than one executable. |


<a name="aurae-runtime-v0-InstanceService"></a>

### InstanceService
TODO Instance Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|


<a name="aurae-runtime-v0-PodService"></a>

### PodService
A pod is a higher level abstraction than Aurae cells, and to most users
/ will look and feel like one or more &#34;containers&#34;.
/
/ Pods will run an OCI compliant container image.
/
/ A pod is a group of one or more containers with shared network and storage.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [PodServiceAllocateRequest](#aurae-runtime-v0-PodServiceAllocateRequest) | [PodServiceAllocateResponse](#aurae-runtime-v0-PodServiceAllocateResponse) |  |
| Start | [PodServiceStartRequest](#aurae-runtime-v0-PodServiceStartRequest) | [PodServiceStartResponse](#aurae-runtime-v0-PodServiceStartResponse) |  |
| Stop | [PodServiceStopRequest](#aurae-runtime-v0-PodServiceStopRequest) | [PodServiceStopResponse](#aurae-runtime-v0-PodServiceStopResponse) |  |
| Free | [PodServiceFreeRequest](#aurae-runtime-v0-PodServiceFreeRequest) | [PodServiceFreeResponse](#aurae-runtime-v0-PodServiceFreeResponse) |  |


<a name="aurae-runtime-v0-SpawnService"></a>

### SpawnService
TODO Spawn Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|

 



## Scalar Value Types

| .proto Type | Notes | C++ | Java | Python | Go | C# | PHP | Ruby |
| ----------- | ----- | --- | ---- | ------ | -- | -- | --- | ---- |
| <a name="double" /> double |  | double | double | float | float64 | double | float | Float |
| <a name="float" /> float |  | float | float | float | float32 | float | float | Float |
| <a name="int32" /> int32 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint32 instead. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="int64" /> int64 | Uses variable-length encoding. Inefficient for encoding negative numbers – if your field is likely to have negative values, use sint64 instead. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="uint32" /> uint32 | Uses variable-length encoding. | uint32 | int | int/long | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="uint64" /> uint64 | Uses variable-length encoding. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum or Fixnum (as required) |
| <a name="sint32" /> sint32 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int32s. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sint64" /> sint64 | Uses variable-length encoding. Signed int value. These more efficiently encode negative numbers than regular int64s. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="fixed32" /> fixed32 | Always four bytes. More efficient than uint32 if values are often greater than 2^28. | uint32 | int | int | uint32 | uint | integer | Bignum or Fixnum (as required) |
| <a name="fixed64" /> fixed64 | Always eight bytes. More efficient than uint64 if values are often greater than 2^56. | uint64 | long | int/long | uint64 | ulong | integer/string | Bignum |
| <a name="sfixed32" /> sfixed32 | Always four bytes. | int32 | int | int | int32 | int | integer | Bignum or Fixnum (as required) |
| <a name="sfixed64" /> sfixed64 | Always eight bytes. | int64 | long | int/long | int64 | long | integer/string | Bignum |
| <a name="bool" /> bool |  | bool | boolean | boolean | bool | bool | boolean | TrueClass/FalseClass |
| <a name="string" /> string | A string must always contain UTF-8 encoded or 7-bit ASCII text. | string | String | str/unicode | string | string | string | String (UTF-8) |
| <a name="bytes" /> bytes | May contain any arbitrary sequence of bytes. | string | ByteString | str | []byte | ByteString | string | String (ASCII-8BIT) |

