# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [observe.proto](#observe-proto)
    - [GetAuraeDaemonLogStreamRequest](#observe-v0-GetAuraeDaemonLogStreamRequest)
    - [GetSubProcessStreamRequest](#observe-v0-GetSubProcessStreamRequest)
    - [LogItem](#observe-v0-LogItem)
  
    - [LogChannelType](#observe-v0-LogChannelType)
  
    - [ObserveService](#observe-v0-ObserveService)
  
- [runtime.proto](#runtime-proto)
    - [Cell](#runtime-v0-Cell)
    - [CellServiceAllocateRequest](#runtime-v0-CellServiceAllocateRequest)
    - [CellServiceAllocateResponse](#runtime-v0-CellServiceAllocateResponse)
    - [CellServiceFreeRequest](#runtime-v0-CellServiceFreeRequest)
    - [CellServiceFreeResponse](#runtime-v0-CellServiceFreeResponse)
    - [CellServiceStartRequest](#runtime-v0-CellServiceStartRequest)
    - [CellServiceStartResponse](#runtime-v0-CellServiceStartResponse)
    - [CellServiceStopRequest](#runtime-v0-CellServiceStopRequest)
    - [CellServiceStopResponse](#runtime-v0-CellServiceStopResponse)
    - [Container](#runtime-v0-Container)
    - [Executable](#runtime-v0-Executable)
    - [Pod](#runtime-v0-Pod)
    - [PodServiceAllocateRequest](#runtime-v0-PodServiceAllocateRequest)
    - [PodServiceAllocateResponse](#runtime-v0-PodServiceAllocateResponse)
    - [PodServiceFreeRequest](#runtime-v0-PodServiceFreeRequest)
    - [PodServiceFreeResponse](#runtime-v0-PodServiceFreeResponse)
    - [PodServiceStartRequest](#runtime-v0-PodServiceStartRequest)
    - [PodServiceStartResponse](#runtime-v0-PodServiceStartResponse)
    - [PodServiceStopRequest](#runtime-v0-PodServiceStopRequest)
    - [PodServiceStopResponse](#runtime-v0-PodServiceStopResponse)
  
    - [CellService](#runtime-v0-CellService)
    - [InstanceService](#runtime-v0-InstanceService)
    - [PodService](#runtime-v0-PodService)
    - [SpawnService](#runtime-v0-SpawnService)
  
- [schedule.proto](#schedule-proto)
- [Scalar Value Types](#scalar-value-types)



<a name="observe-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## observe.proto



<a name="observe-v0-GetAuraeDaemonLogStreamRequest"></a>

### GetAuraeDaemonLogStreamRequest







<a name="observe-v0-GetSubProcessStreamRequest"></a>

### GetSubProcessStreamRequest
TODO: not implemented


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel_type | [LogChannelType](#observe-v0-LogChannelType) |  |  |
| process_id | [int64](#int64) |  |  |






<a name="observe-v0-LogItem"></a>

### LogItem



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel | [string](#string) |  |  |
| line | [string](#string) |  |  |
| timestamp | [int64](#int64) |  |  |





 


<a name="observe-v0-LogChannelType"></a>

### LogChannelType


| Name | Number | Description |
| ---- | ------ | ----------- |
| LOG_CHANNEL_TYPE_UNSPECIFIED | 0 |  |
| LOG_CHANNEL_TYPE_STDOUT | 1 |  |
| LOG_CHANNEL_TYPE_STDERR | 2 |  |


 

 


<a name="observe-v0-ObserveService"></a>

### ObserveService


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| GetAuraeDaemonLogStream | [GetAuraeDaemonLogStreamRequest](#observe-v0-GetAuraeDaemonLogStreamRequest) | [LogItem](#observe-v0-LogItem) stream | request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). |
| GetSubProcessStream | [GetSubProcessStreamRequest](#observe-v0-GetSubProcessStreamRequest) | [LogItem](#observe-v0-LogItem) stream | TODO: request log stream for a sub process |

 



<a name="runtime-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## runtime.proto



<a name="runtime-v0-Cell"></a>

### Cell
An isolation resource used to divide a system into smaller resource
/ boundaries.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | Resource parameters for control groups (cgroups) / Build on the [cgroups-rs](https://github.com/kata-containers/cgroups-rs) / crate. See / [examples](https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs) |
| cpu_cpus | [string](#string) |  | A comma-separated list of CPU IDs where the task in the control group / can run. Dashes between numbers indicate ranges. |
| cpu_shares | [uint64](#uint64) |  | Cgroups can be guaranteed a minimum number of &#34;CPU shares&#34; / when a system is busy. This does not limit a cgroup&#39;s CPU / usage if the CPUs are not busy. For further information, / see Documentation/scheduler/sched-design-CFS.rst (or / Documentation/scheduler/sched-design-CFS.txt in Linux 5.2 / and earlier). / / Weight of how much of the total CPU time should this control / group get. Note that this is hierarchical, so this is weighted / against the siblings of this control group. |
| cpu_mems | [string](#string) |  | Same syntax as the cpus field of this structure, but applies to / memory nodes instead of processors. |
| cpu_quota | [int64](#int64) |  | In one period, how much can the tasks run in microseconds. |
| ns_share_mount | [bool](#bool) |  | Linux namespaces to share with the calling process. / If all values are set to false, the resulting cell / will be as isolated as possible. / / Each shared namespace is a potential security risk. |
| ns_share_uts | [bool](#bool) |  |  |
| ns_share_ipc | [bool](#bool) |  |  |
| ns_share_pid | [bool](#bool) |  |  |
| ns_share_net | [bool](#bool) |  |  |
| ns_share_cgroup | [bool](#bool) |  |  |






<a name="runtime-v0-CellServiceAllocateRequest"></a>

### CellServiceAllocateRequest
An Aurae cell is a name given to Linux control groups (cgroups) that also include
/ a name, and special pre-exec functionality that is executed from within the same context
/ as any executables scheduled.
/
/ A cell must be allocated for every executable scheduled. A cell defines the resource
/ constraints of the system to allocate for an arbitrary use case.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell | [Cell](#runtime-v0-Cell) |  | A smaller resource constrained section of the system. |






<a name="runtime-v0-CellServiceAllocateResponse"></a>

### CellServiceAllocateResponse
The response after a cell has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| cgroup_v2 | [bool](#bool) |  | A bool that will be set to true if the cgroup was created with / cgroup v2 controller. |






<a name="runtime-v0-CellServiceFreeRequest"></a>

### CellServiceFreeRequest
Used to remove or free a cell after it has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |






<a name="runtime-v0-CellServiceFreeResponse"></a>

### CellServiceFreeResponse
Response after removing or freeing a cell.






<a name="runtime-v0-CellServiceStartRequest"></a>

### CellServiceStartRequest
A request for starting an executable inside of a Cell.
/
/ This is the lowest level of raw executive functionality.
/ Here you can define shell commands, and meta information about the command.
/ An executable is started synchronously.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable | [Executable](#runtime-v0-Executable) |  |  |






<a name="runtime-v0-CellServiceStartResponse"></a>

### CellServiceStartResponse
The response after starting an executable within a Cell.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pid | [int32](#int32) |  | Return a pid as an int32 based on the pid_t type / in various libc libraries. |






<a name="runtime-v0-CellServiceStopRequest"></a>

### CellServiceStopRequest
Request to stop an executable at runtime.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable_name | [string](#string) |  |  |






<a name="runtime-v0-CellServiceStopResponse"></a>

### CellServiceStopResponse







<a name="runtime-v0-Container"></a>

### Container







<a name="runtime-v0-Executable"></a>

### Executable
The most primitive workload in Aurae, a standard executable process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |
| command | [string](#string) |  |  |
| description | [string](#string) |  |  |






<a name="runtime-v0-Pod"></a>

### Pod







<a name="runtime-v0-PodServiceAllocateRequest"></a>

### PodServiceAllocateRequest







<a name="runtime-v0-PodServiceAllocateResponse"></a>

### PodServiceAllocateResponse







<a name="runtime-v0-PodServiceFreeRequest"></a>

### PodServiceFreeRequest







<a name="runtime-v0-PodServiceFreeResponse"></a>

### PodServiceFreeResponse







<a name="runtime-v0-PodServiceStartRequest"></a>

### PodServiceStartRequest







<a name="runtime-v0-PodServiceStartResponse"></a>

### PodServiceStartResponse







<a name="runtime-v0-PodServiceStopRequest"></a>

### PodServiceStopRequest







<a name="runtime-v0-PodServiceStopResponse"></a>

### PodServiceStopResponse






 

 

 


<a name="runtime-v0-CellService"></a>

### CellService
Cells is the most fundamental isolation boundary for Aurae.
/ A cell is an isolate set of resources of the system which can be
/ used to run workloads.
/
/ A cell is composed of a unique cgroup namespace, and unshared kernel
/ namespaces.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [CellServiceAllocateRequest](#runtime-v0-CellServiceAllocateRequest) | [CellServiceAllocateResponse](#runtime-v0-CellServiceAllocateResponse) | Reserve requested system resources for a new cell. / For cells specifically this will allocate and reserve cgroup resources / only. |
| Free | [CellServiceFreeRequest](#runtime-v0-CellServiceFreeRequest) | [CellServiceFreeResponse](#runtime-v0-CellServiceFreeResponse) | Free up previously requested resources for an existing cell |
| Start | [CellServiceStartRequest](#runtime-v0-CellServiceStartRequest) | [CellServiceStartResponse](#runtime-v0-CellServiceStartResponse) | Start a new Executable inside of an existing cell. Can be called / in serial to start more than one executable in the same cell. |
| Stop | [CellServiceStopRequest](#runtime-v0-CellServiceStopRequest) | [CellServiceStopResponse](#runtime-v0-CellServiceStopResponse) | Stop one or more Executables inside of an existing cell. / Can be called in serial to stop/retry more than one executable. |


<a name="runtime-v0-InstanceService"></a>

### InstanceService
TODO Instance Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|


<a name="runtime-v0-PodService"></a>

### PodService
A pod is a higher level abstraction than Aurae cells, and to most users
/ will look at feel like one or more &#34;containers&#34;.
/
/ Pods will run an OCI compliant container image.
/
/ A pod is a group of one or more containers with shared network and storage.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [PodServiceAllocateRequest](#runtime-v0-PodServiceAllocateRequest) | [PodServiceAllocateResponse](#runtime-v0-PodServiceAllocateResponse) |  |
| Start | [PodServiceStartRequest](#runtime-v0-PodServiceStartRequest) | [PodServiceStartResponse](#runtime-v0-PodServiceStartResponse) |  |
| Stop | [PodServiceStopRequest](#runtime-v0-PodServiceStopRequest) | [PodServiceStopResponse](#runtime-v0-PodServiceStopResponse) |  |
| Free | [PodServiceFreeRequest](#runtime-v0-PodServiceFreeRequest) | [PodServiceFreeResponse](#runtime-v0-PodServiceFreeResponse) |  |


<a name="runtime-v0-SpawnService"></a>

### SpawnService
TODO Spawn Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|

 



<a name="schedule-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## schedule.proto


 

 

 

 



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

