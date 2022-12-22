# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [observe.proto](#observe-proto)
    - [GetAuraeDaemonLogStreamRequest](#observe-GetAuraeDaemonLogStreamRequest)
    - [GetSubProcessStreamRequest](#observe-GetSubProcessStreamRequest)
    - [LogItem](#observe-LogItem)
  
    - [LogChannelType](#observe-LogChannelType)
  
    - [Observe](#observe-Observe)
  
- [runtime.proto](#runtime-proto)
    - [AllocateCellRequest](#runtime-AllocateCellRequest)
    - [AllocateCellResponse](#runtime-AllocateCellResponse)
    - [Cell](#runtime-Cell)
    - [Executable](#runtime-Executable)
    - [FreeCellRequest](#runtime-FreeCellRequest)
    - [FreeCellResponse](#runtime-FreeCellResponse)
    - [StartExecutableRequest](#runtime-StartExecutableRequest)
    - [StartExecutableResponse](#runtime-StartExecutableResponse)
    - [StopExecutableRequest](#runtime-StopExecutableRequest)
    - [StopExecutableResponse](#runtime-StopExecutableResponse)
  
    - [CellService](#runtime-CellService)
    - [Instances](#runtime-Instances)
    - [Pods](#runtime-Pods)
    - [Spawn](#runtime-Spawn)
  
- [schedule.proto](#schedule-proto)
- [Scalar Value Types](#scalar-value-types)



<a name="observe-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## observe.proto



<a name="observe-GetAuraeDaemonLogStreamRequest"></a>

### GetAuraeDaemonLogStreamRequest







<a name="observe-GetSubProcessStreamRequest"></a>

### GetSubProcessStreamRequest
TODO: not implemented


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel_type | [LogChannelType](#observe-LogChannelType) |  |  |
| process_id | [int64](#int64) |  |  |






<a name="observe-LogItem"></a>

### LogItem



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel | [string](#string) |  |  |
| line | [string](#string) |  |  |
| timestamp | [int64](#int64) |  |  |





 


<a name="observe-LogChannelType"></a>

### LogChannelType


| Name | Number | Description |
| ---- | ------ | ----------- |
| CHANNEL_STDOUT | 0 |  |
| CHANNEL_STDERR | 1 |  |


 

 


<a name="observe-Observe"></a>

### Observe


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| GetAuraeDaemonLogStream | [GetAuraeDaemonLogStreamRequest](#observe-GetAuraeDaemonLogStreamRequest) | [LogItem](#observe-LogItem) stream | request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). |
| GetSubProcessStream | [GetSubProcessStreamRequest](#observe-GetSubProcessStreamRequest) | [LogItem](#observe-LogItem) stream | TODO: request log stream for a sub process |

 



<a name="runtime-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## runtime.proto



<a name="runtime-AllocateCellRequest"></a>

### AllocateCellRequest
An Aurae cell is a name given to Linux control groups (cgroups) that also include
/ a name, and special pre-exec functionality that is executed from within the same context
/ as any executables scheduled.
/
/ A cell must be allocated for every executable scheduled. A cell defines the resource
/ constraints of the system to allocate for an arbitrary use case.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell | [Cell](#runtime-Cell) |  | A smaller resource constrained section of the system. |






<a name="runtime-AllocateCellResponse"></a>

### AllocateCellResponse
The response after a cell has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| cgroup_v2 | [bool](#bool) |  | A bool that will be set to true if the cgroup was created with / cgroup v2 controller. |






<a name="runtime-Cell"></a>

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






<a name="runtime-Executable"></a>

### Executable
The most primitive workload in Aurae, a standard executable process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |
| command | [string](#string) |  |  |
| args | [string](#string) | repeated |  |
| description | [string](#string) |  |  |






<a name="runtime-FreeCellRequest"></a>

### FreeCellRequest
Used to remove or free a cell after it has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |






<a name="runtime-FreeCellResponse"></a>

### FreeCellResponse
Response after removing or freeing a cell.






<a name="runtime-StartExecutableRequest"></a>

### StartExecutableRequest
A request for starting an executable inside of a Cell.
/
/ This is the lowest level of raw executive functionality.
/ Here you can define shell commands, and meta information about the command.
/ An executable is started synchronously.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable | [Executable](#runtime-Executable) |  |  |






<a name="runtime-StartExecutableResponse"></a>

### StartExecutableResponse
The response after starting an executable within a Cell.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pid | [int32](#int32) |  | Return a pid as an int32 based on the pid_t type / in various libc libraries. |






<a name="runtime-StopExecutableRequest"></a>

### StopExecutableRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| executable_name | [string](#string) |  |  |






<a name="runtime-StopExecutableResponse"></a>

### StopExecutableResponse






 

 

 


<a name="runtime-CellService"></a>

### CellService
Cells is the most fundamental isolation boundary for Aurae.
/ A cell is an isolate set of resources of the system which can be
/ used to run workloads.
/
/ A cell is composed of a unique cgroup namespace, and unshared kernel
/ namespaces.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [AllocateCellRequest](#runtime-AllocateCellRequest) | [AllocateCellResponse](#runtime-AllocateCellResponse) | Reserve requested system resources for a new cell. / For cells specifically this will allocate and reserve cgroup resources / only. |
| Free | [FreeCellRequest](#runtime-FreeCellRequest) | [FreeCellResponse](#runtime-FreeCellResponse) | Free up previously requested resources for an existing cell |
| Start | [StartExecutableRequest](#runtime-StartExecutableRequest) | [StartExecutableResponse](#runtime-StartExecutableResponse) | Start a new Executable inside of an existing cell. Can be called / in serial to start more than one executable in the same cell. |
| Stop | [StopExecutableRequest](#runtime-StopExecutableRequest) | [StopExecutableResponse](#runtime-StopExecutableResponse) | Stop one or more Executables inside of an existing cell. / Can be called in serial to stop/retry more than one executable. |


<a name="runtime-Instances"></a>

### Instances
TODO Instances Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|


<a name="runtime-Pods"></a>

### Pods
TODO Pods Service

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|


<a name="runtime-Spawn"></a>

### Spawn
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

