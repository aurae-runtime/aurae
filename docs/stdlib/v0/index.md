# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [meta.proto](#meta-proto)
    - [AuraeMeta](#meta-AuraeMeta)
    - [ProcessMeta](#meta-ProcessMeta)
  
    - [Status](#meta-Status)
  
- [observe.proto](#observe-proto)
    - [GetAuraeDaemonLogStreamRequest](#observe-GetAuraeDaemonLogStreamRequest)
    - [GetSubProcessStreamRequest](#observe-GetSubProcessStreamRequest)
    - [LogItem](#observe-LogItem)
    - [StatusRequest](#observe-StatusRequest)
    - [StatusResponse](#observe-StatusResponse)
  
    - [LogChannelType](#observe-LogChannelType)
  
    - [Observe](#observe-Observe)
  
- [runtime.proto](#runtime-proto)
    - [Cell](#runtime-Cell)
    - [CellStatus](#runtime-CellStatus)
    - [Container](#runtime-Container)
    - [ContainerStatus](#runtime-ContainerStatus)
    - [Executable](#runtime-Executable)
    - [ExecutableStatus](#runtime-ExecutableStatus)
    - [Instance](#runtime-Instance)
    - [Pod](#runtime-Pod)
    - [PodStatus](#runtime-PodStatus)
    - [SpawnRequest](#runtime-SpawnRequest)
    - [SpawnResponse](#runtime-SpawnResponse)
    - [VirtualMachine](#runtime-VirtualMachine)
    - [VirtualMachineStatus](#runtime-VirtualMachineStatus)
  
    - [Core](#runtime-Core)
  
- [schedule.proto](#schedule-proto)
    - [ExecutableDestroyResponse](#schedule-ExecutableDestroyResponse)
    - [ExecutableDisableResponse](#schedule-ExecutableDisableResponse)
    - [ExecutableEnableResponse](#schedule-ExecutableEnableResponse)
    - [ShowDisabledRequest](#schedule-ShowDisabledRequest)
    - [ShowDisabledResponse](#schedule-ShowDisabledResponse)
    - [ShowEnabledRequest](#schedule-ShowEnabledRequest)
    - [ShowEnabledResponse](#schedule-ShowEnabledResponse)
  
    - [Schedule](#schedule-Schedule)
    - [ScheduleExecutable](#schedule-ScheduleExecutable)
  
- [Scalar Value Types](#scalar-value-types)



<a name="meta-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## meta.proto



<a name="meta-AuraeMeta"></a>

### AuraeMeta



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| name | [string](#string) |  |  |
| message | [string](#string) |  |  |






<a name="meta-ProcessMeta"></a>

### ProcessMeta



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| pid | [int32](#int32) |  |  |





 


<a name="meta-Status"></a>

### Status
Status represents the state of an object within Aurae.
/ The status Enum has special meaning used for each value.

| Name | Number | Description |
| ---- | ------ | ----------- |
| STATUS_UNKNOWN | 0 | Unknown denotes a rogue status, and should only be used for emergencies or development. Generally speaking Aurae / should never have an unknown object unless something has gone very, very wrong. |
| STATUS_STANDBY | 1 | Standby denotes an object that is healthy but not active. Something that has passed any preliminary or prerequisite steps but is not actively executing or running. Standby is a synonym for &#34;enabled&#34;. |
| STATUS_ACTIVE | 3 | Active denotes an object that is currently active. The object is currently executing at the point in time the / request was issued. |
| STATUS_PASSIVE | 4 | Passive is the opposite of standby. The object is registered but is disabled and has not gone through any / preliminary or prerequisite steps. Passive is a synonym for &#34;disabled&#34;. |
| STATUS_ERROR | 5 | Error denotes a failure, but not severity. Something has gone wrong, there will be more information elsewhere. |
| STATUS_COMPLETE | 6 | Complete denotes that an action is complete and no longer active. |


 

 

 



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






<a name="observe-StatusRequest"></a>

### StatusRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="observe-StatusResponse"></a>

### StatusResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |





 


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
| Status | [StatusRequest](#observe-StatusRequest) | [StatusResponse](#observe-StatusResponse) |  |
| GetAuraeDaemonLogStream | [GetAuraeDaemonLogStreamRequest](#observe-GetAuraeDaemonLogStreamRequest) | [LogItem](#observe-LogItem) stream | request log stream for aurae. everything logged via log macros in aurae (info!, error!, trace!, ... ). |
| GetSubProcessStream | [GetSubProcessStreamRequest](#observe-GetSubProcessStreamRequest) | [LogItem](#observe-LogItem) stream | TODO: request log stream for a sub process |

 



<a name="runtime-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## runtime.proto
The runtime subsystem is a synchronous and stateless subsystem which
enables the most fundamental of runtime functionality.

The runtime subsystem is the lowest level of Aurae. Here the runtime
procedures can be executed in parallel. Each of the procedures defines a
core paradigm of work.


<a name="runtime-Cell"></a>

### Cell
A set of containers running on the same kernel as the auraed responding to the request.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| containers | [Container](#runtime-Container) | repeated | A set of containers. |






<a name="runtime-CellStatus"></a>

### CellStatus



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |






<a name="runtime-Container"></a>

### Container
Container represents is an OCI compliant container image which can be executed.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| image | [string](#string) |  | OCI compliant image. |






<a name="runtime-ContainerStatus"></a>

### ContainerStatus
ContainerStatus is the status of a container after it has been executed.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| proc | [meta.ProcessMeta](#meta-ProcessMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |






<a name="runtime-Executable"></a>

### Executable
Executable is the lowest level of compute that Aurae can execute. A basic process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| command | [string](#string) |  | Command resembles systemd&#39;s ExecStart. This is the shell command (with arguments) you intend to execute. |
| comment | [string](#string) |  | Comment is an arbitrary (user defined) comment used to identify the Executable at runtime. |






<a name="runtime-ExecutableStatus"></a>

### ExecutableStatus
ExecutableStatus is only returned after a process completes. Because runtime is a synchronous subsystem
this will only return upon a terminated process.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| proc | [meta.ProcessMeta](#meta-ProcessMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |
| stdout | [string](#string) |  | The full stdout data. |
| stderr | [string](#string) |  | The full stderr data. |
| exit_code | [string](#string) |  | The exit code (return code) of the process after termination. |






<a name="runtime-Instance"></a>

### Instance
Instance is a recursive graph structure which holds the meta data for the nested Aurae instances
running on a machine.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| is_root | [bool](#bool) |  |  |
| nested_instances | [Instance](#runtime-Instance) | repeated |  |






<a name="runtime-Pod"></a>

### Pod
Pod is a group of containers running in a spawned Aurae instance.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| cells | [Cell](#runtime-Cell) | repeated | The cells to create within the spawned Aurae instance. |






<a name="runtime-PodStatus"></a>

### PodStatus
PodStatus is the status of a completed pod and its subsequent containers.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |
| spawn_response | [SpawnResponse](#runtime-SpawnResponse) |  | The response of the necessary spawn for a pod. |
| instance | [Instance](#runtime-Instance) |  | If the spawn is successful, the nested Aurae instance. |
| cell_status | [CellStatus](#runtime-CellStatus) |  | If the cell creation is successful, the cell of containers running in the spawned Aurae instance. |






<a name="runtime-SpawnRequest"></a>

### SpawnRequest
Parameters for spawning a new nested Aurae instance.






<a name="runtime-SpawnResponse"></a>

### SpawnResponse
Response of a spawn.






<a name="runtime-VirtualMachine"></a>

### VirtualMachine
A long lived virtual machine which will persist on termination.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| name | [string](#string) |  |  |
| image | [string](#string) |  |  |






<a name="runtime-VirtualMachineStatus"></a>

### VirtualMachineStatus
Status of a terminated virtual machine.


| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |





 

 

 


<a name="runtime-Core"></a>

### Core
Core is a synchronous subsystem which defines the main methods for executing and starting
workloads within an Aurae system.

Core is designed to be called by higher order instance of Aurae, clients, or higher order
systems in general.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| RunExecutable | [Executable](#runtime-Executable) | [ExecutableStatus](#runtime-ExecutableStatus) | Run an Executable, the most fundamental runtime process. Akin to shell executing a command.

* Executable should exist on system, does not pull from a remote. |
| RunPod | [Pod](#runtime-Pod) | [PodStatus](#runtime-PodStatus) | Spawn a new instance of Aurae, and then create a cell of containers inside the nested Aurae instance.

* Accepts an OCI compliant container image, will always pull the image before running. * Assumes Spawn() is successful and is able to bridge to the nested Aurae and the nested cell. |
| Spawn | [SpawnRequest](#runtime-SpawnRequest) | [SpawnResponse](#runtime-SpawnResponse) | Spawn a short lived (ephemeral) nested virtual instance of Aurae which will terminate on exit. Akin to fork() in Linux, each nested instance inherits properties from the parent but runs an isolated virtual machine with its own kernel and auraed instance.

* Inherits a bridged network device from the parent (TAP). * Accepts an OCI compliant container image, will always pull the image before running. * Manages mTLS certificates as necessary. |
| RunVirtualMachine | [VirtualMachine](#runtime-VirtualMachine) | [VirtualMachineStatus](#runtime-VirtualMachineStatus) | Run a long lived virtual instance which will persist on exit. Akin to a QEMU virtual machine running with the base auraed as a hypervisor.

* Pull from a remote registry (e.g. qcow format) * Accepts a qcow compliant virtual machine image, will always pull the image before running. |
| RunCell | [Cell](#runtime-Cell) | [CellStatus](#runtime-CellStatus) | Run a set of containers in a unique Cgroup with shared namespaces. Akin to running a &#34;container&#34; in its most native way.

* Accepts an OCI compliant container image, will always pull the image before running. * Runs directly on the same kernel as the root Auraed. |

 



<a name="schedule-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## schedule.proto



<a name="schedule-ExecutableDestroyResponse"></a>

### ExecutableDestroyResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="schedule-ExecutableDisableResponse"></a>

### ExecutableDisableResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="schedule-ExecutableEnableResponse"></a>

### ExecutableEnableResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="schedule-ShowDisabledRequest"></a>

### ShowDisabledRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="schedule-ShowDisabledResponse"></a>

### ShowDisabledResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| Executables | [runtime.Executable](#runtime-Executable) | repeated |  |






<a name="schedule-ShowEnabledRequest"></a>

### ShowEnabledRequest



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="schedule-ShowEnabledResponse"></a>

### ShowEnabledResponse



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| Executables | [runtime.Executable](#runtime-Executable) | repeated |  |





 

 

 


<a name="schedule-Schedule"></a>

### Schedule


| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| ShowEnabled | [ShowEnabledRequest](#schedule-ShowEnabledRequest) | [ShowEnabledResponse](#schedule-ShowEnabledResponse) | ShowEnabled will return a response of everything enabled on a system |
| ShowDisabled | [ShowDisabledRequest](#schedule-ShowDisabledRequest) | [ShowDisabledResponse](#schedule-ShowDisabledResponse) | ShowDisabled will return a response of everything disabled on a system |


<a name="schedule-ScheduleExecutable"></a>

### ScheduleExecutable
We break ScheduleExecutable out into its own subsystem for authz purposes

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Enable | [.runtime.Executable](#runtime-Executable) | [ExecutableEnableResponse](#schedule-ExecutableEnableResponse) |  |
| Disable | [.runtime.Executable](#runtime-Executable) | [ExecutableDisableResponse](#schedule-ExecutableDisableResponse) |  |
| Destroy | [.runtime.Executable](#runtime-Executable) | [ExecutableDestroyResponse](#schedule-ExecutableDestroyResponse) |  |

 



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

