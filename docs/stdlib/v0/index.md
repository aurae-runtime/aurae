# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [cells.proto](#cells-proto)
    - [Cell](#aurae-cells-v0-Cell)
    - [CellGraphNode](#aurae-cells-v0-CellGraphNode)
    - [CellServiceAllocateRequest](#aurae-cells-v0-CellServiceAllocateRequest)
    - [CellServiceAllocateResponse](#aurae-cells-v0-CellServiceAllocateResponse)
    - [CellServiceFreeRequest](#aurae-cells-v0-CellServiceFreeRequest)
    - [CellServiceFreeResponse](#aurae-cells-v0-CellServiceFreeResponse)
    - [CellServiceListRequest](#aurae-cells-v0-CellServiceListRequest)
    - [CellServiceListResponse](#aurae-cells-v0-CellServiceListResponse)
    - [CellServiceStartRequest](#aurae-cells-v0-CellServiceStartRequest)
    - [CellServiceStartResponse](#aurae-cells-v0-CellServiceStartResponse)
    - [CellServiceStopRequest](#aurae-cells-v0-CellServiceStopRequest)
    - [CellServiceStopResponse](#aurae-cells-v0-CellServiceStopResponse)
    - [CpuController](#aurae-cells-v0-CpuController)
    - [CpusetController](#aurae-cells-v0-CpusetController)
    - [Executable](#aurae-cells-v0-Executable)
  
    - [CellService](#aurae-cells-v0-CellService)
  
- [discovery.proto](#discovery-proto)
    - [DiscoverRequest](#aurae-discovery-v0-DiscoverRequest)
    - [DiscoverResponse](#aurae-discovery-v0-DiscoverResponse)
  
    - [DiscoveryService](#aurae-discovery-v0-DiscoveryService)
  
- [observe.proto](#observe-proto)
    - [GetAuraeDaemonLogStreamRequest](#aurae-observe-v0-GetAuraeDaemonLogStreamRequest)
    - [GetAuraeDaemonLogStreamResponse](#aurae-observe-v0-GetAuraeDaemonLogStreamResponse)
    - [GetPosixSignalsStreamRequest](#aurae-observe-v0-GetPosixSignalsStreamRequest)
    - [GetPosixSignalsStreamResponse](#aurae-observe-v0-GetPosixSignalsStreamResponse)
    - [GetSubProcessStreamRequest](#aurae-observe-v0-GetSubProcessStreamRequest)
    - [GetSubProcessStreamResponse](#aurae-observe-v0-GetSubProcessStreamResponse)
    - [LogItem](#aurae-observe-v0-LogItem)
    - [Signal](#aurae-observe-v0-Signal)
  
    - [LogChannelType](#aurae-observe-v0-LogChannelType)
  
    - [ObserveService](#aurae-observe-v0-ObserveService)
  
- [Scalar Value Types](#scalar-value-types)



<a name="cells-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## cells.proto



<a name="aurae-cells-v0-Cell"></a>

### Cell
An isolation resource used to divide a system into smaller resource
/ boundaries.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  | Resource parameters for control groups (cgroups) / Build on the [cgroups-rs](https://github.com/kata-containers/cgroups-rs) / crate. See / [examples](https://github.com/kata-containers/cgroups-rs/blob/main/tests/builder.rs) |
| cpu | [CpuController](#aurae-cells-v0-CpuController) |  |  |
| cpuset | [CpusetController](#aurae-cells-v0-CpusetController) |  |  |
| isolate_process | [bool](#bool) |  | Will isolate the process (and proc filesystem) from the host. / Will unshare the pid, ipc, uts, and mount namespaces. / The cgroup namespace is always unshared with the host. / / Default: false |
| isolate_network | [bool](#bool) |  | Will isolate the network from the host. / Will unshare the net namespaces. / The cgroup namespace is always unshared with the host. / / Default: false |






<a name="aurae-cells-v0-CellGraphNode"></a>

### CellGraphNode



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell | [Cell](#aurae-cells-v0-Cell) |  |  |
| children | [CellGraphNode](#aurae-cells-v0-CellGraphNode) | repeated |  |






<a name="aurae-cells-v0-CellServiceAllocateRequest"></a>

### CellServiceAllocateRequest
An Aurae cell is a name given to Linux control groups (cgroups) that also include
/ a name, and special pre-exec functionality that is executed from within the same context
/ as any executables scheduled.
/
/ A cell must be allocated for every executable scheduled. A cell defines the resource
/ constraints of the system to allocate for an arbitrary use case.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell | [Cell](#aurae-cells-v0-Cell) |  | A smaller resource constrained section of the system. |






<a name="aurae-cells-v0-CellServiceAllocateResponse"></a>

### CellServiceAllocateResponse
The response after a cell has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |
| cgroup_v2 | [bool](#bool) |  | A bool that will be set to true if the cgroup was created with / cgroup v2 controller. |






<a name="aurae-cells-v0-CellServiceFreeRequest"></a>

### CellServiceFreeRequest
Used to remove or free a cell after it has been allocated.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) |  |  |






<a name="aurae-cells-v0-CellServiceFreeResponse"></a>

### CellServiceFreeResponse
Response after removing or freeing a cell.






<a name="aurae-cells-v0-CellServiceListRequest"></a>

### CellServiceListRequest







<a name="aurae-cells-v0-CellServiceListResponse"></a>

### CellServiceListResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cells | [CellGraphNode](#aurae-cells-v0-CellGraphNode) | repeated |  |






<a name="aurae-cells-v0-CellServiceStartRequest"></a>

### CellServiceStartRequest
A request for starting an executable inside of a Cell.
/
/ This is the lowest level of raw executive functionality.
/ Here you can define shell commands, and meta information about the command.
/ An executable is started synchronously.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) | optional |  |
| executable | [Executable](#aurae-cells-v0-Executable) |  |  |






<a name="aurae-cells-v0-CellServiceStartResponse"></a>

### CellServiceStartResponse
The response after starting an executable within a Cell.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pid | [int32](#int32) |  | Return a pid as an int32 based on the pid_t type / in various libc libraries. |






<a name="aurae-cells-v0-CellServiceStopRequest"></a>

### CellServiceStopRequest
Request to stop an executable at runtime.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cell_name | [string](#string) | optional |  |
| executable_name | [string](#string) |  |  |






<a name="aurae-cells-v0-CellServiceStopResponse"></a>

### CellServiceStopResponse







<a name="aurae-cells-v0-CpuController"></a>

### CpuController
Docs: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpu


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| weight | [uint64](#uint64) | optional | Weight of how much of the total CPU time should this control group get. Note that this is hierarchical, so this is weighted against the siblings of this control group.

* Minimum: 1 * Maximum: 10_000 |
| max | [int64](#int64) | optional | In one period (1_000_000), how much can the tasks run.

* Minimum: 0

By default a cgroup has no limit, represented as the literal string &#34;max&#34;. Not settings this field retains the default of no limit. |






<a name="aurae-cells-v0-CpusetController"></a>

### CpusetController
Docs: https://docs.kernel.org/admin-guide/cgroup-v2.html#cpuset


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| cpus | [string](#string) | optional | A comma-separated list of CPU IDs where the task in the control group can run. Dashes between numbers indicate ranges. |
| mems | [string](#string) | optional | Same syntax as the cpus field of this structure, but applies to memory nodes instead of processors. |






<a name="aurae-cells-v0-Executable"></a>

### Executable
The most primitive workload in Aurae, a standard executable process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |
| command | [string](#string) |  |  |
| description | [string](#string) |  |  |





 

 

 


<a name="aurae-cells-v0-CellService"></a>

### CellService
Cells is the most fundamental isolation boundary for Aurae.
/ A cell is an isolate set of resources of the system which can be
/ used to run workloads.
/
/ A cell is composed of a unique cgroup namespace, and unshared kernel
/ namespaces.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [CellServiceAllocateRequest](#aurae-cells-v0-CellServiceAllocateRequest) | [CellServiceAllocateResponse](#aurae-cells-v0-CellServiceAllocateResponse) | Reserve requested system resources for a new cell. / For cells specifically this will allocate and reserve cgroup resources / only. |
| Free | [CellServiceFreeRequest](#aurae-cells-v0-CellServiceFreeRequest) | [CellServiceFreeResponse](#aurae-cells-v0-CellServiceFreeResponse) | Free up previously requested resources for an existing cell |
| Start | [CellServiceStartRequest](#aurae-cells-v0-CellServiceStartRequest) | [CellServiceStartResponse](#aurae-cells-v0-CellServiceStartResponse) | Start a new Executable inside of an existing cell. Can be called / in serial to start more than one executable in the same cell. |
| Stop | [CellServiceStopRequest](#aurae-cells-v0-CellServiceStopRequest) | [CellServiceStopResponse](#aurae-cells-v0-CellServiceStopResponse) | Stop one or more Executables inside of an existing cell. / Can be called in serial to stop/retry more than one executable. |
| List | [CellServiceListRequest](#aurae-cells-v0-CellServiceListRequest) | [CellServiceListResponse](#aurae-cells-v0-CellServiceListResponse) |  |

 



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







<a name="aurae-observe-v0-GetAuraeDaemonLogStreamResponse"></a>

### GetAuraeDaemonLogStreamResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| item | [LogItem](#aurae-observe-v0-LogItem) |  |  |






<a name="aurae-observe-v0-GetPosixSignalsStreamRequest"></a>

### GetPosixSignalsStreamRequest







<a name="aurae-observe-v0-GetPosixSignalsStreamResponse"></a>

### GetPosixSignalsStreamResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| signal | [Signal](#aurae-observe-v0-Signal) |  |  |






<a name="aurae-observe-v0-GetSubProcessStreamRequest"></a>

### GetSubProcessStreamRequest
TODO: not implemented


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel_type | [LogChannelType](#aurae-observe-v0-LogChannelType) |  |  |
| process_id | [int64](#int64) |  |  |






<a name="aurae-observe-v0-GetSubProcessStreamResponse"></a>

### GetSubProcessStreamResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| item | [LogItem](#aurae-observe-v0-LogItem) |  |  |






<a name="aurae-observe-v0-LogItem"></a>

### LogItem



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| channel | [string](#string) |  |  |
| line | [string](#string) |  |  |
| timestamp | [int64](#int64) |  |  |






<a name="aurae-observe-v0-Signal"></a>

### Signal



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| signal | [int32](#int32) |  |  |
| process_id | [int64](#int64) |  |  |





 


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
| GetAuraeDaemonLogStream | [GetAuraeDaemonLogStreamRequest](#aurae-observe-v0-GetAuraeDaemonLogStreamRequest) | [GetAuraeDaemonLogStreamResponse](#aurae-observe-v0-GetAuraeDaemonLogStreamResponse) stream | request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). |
| GetSubProcessStream | [GetSubProcessStreamRequest](#aurae-observe-v0-GetSubProcessStreamRequest) | [GetSubProcessStreamResponse](#aurae-observe-v0-GetSubProcessStreamResponse) stream | TODO: request log stream for a sub process |
| GetPosixSignalsStream | [GetPosixSignalsStreamRequest](#aurae-observe-v0-GetPosixSignalsStreamRequest) | [GetPosixSignalsStreamResponse](#aurae-observe-v0-GetPosixSignalsStreamResponse) stream | request POSIX signals stream for the host |

 



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

