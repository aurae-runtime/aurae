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
    - [Container](#runtime-Container)
    - [ContainerStatus](#runtime-ContainerStatus)
    - [Executable](#runtime-Executable)
    - [ExecutableStatus](#runtime-ExecutableStatus)
    - [Instance](#runtime-Instance)
    - [InstanceMeta](#runtime-InstanceMeta)
    - [InstanceMetaStatus](#runtime-InstanceMetaStatus)
    - [InstanceStatus](#runtime-InstanceStatus)
  
    - [Runtime](#runtime-Runtime)
  
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



<a name="runtime-Container"></a>

### Container



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| name | [string](#string) |  |  |
| image | [string](#string) |  |  |






<a name="runtime-ContainerStatus"></a>

### ContainerStatus



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| proc | [meta.ProcessMeta](#meta-ProcessMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |






<a name="runtime-Executable"></a>

### Executable



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| command | [string](#string) |  | Command resembles systemd&#39;s ExecStart. This is the shell command (with arguments) you intend to execute. |
| comment | [string](#string) |  | Comment is an arbitrary (user defined) comment used to identify the Executable at runtime. |






<a name="runtime-ExecutableStatus"></a>

### ExecutableStatus



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| proc | [meta.ProcessMeta](#meta-ProcessMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |
| stdout | [string](#string) |  |  |
| stderr | [string](#string) |  |  |
| exit_code | [string](#string) |  |  |






<a name="runtime-Instance"></a>

### Instance



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| name | [string](#string) |  |  |
| image | [string](#string) |  |  |






<a name="runtime-InstanceMeta"></a>

### InstanceMeta



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |






<a name="runtime-InstanceMetaStatus"></a>

### InstanceMetaStatus



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |






<a name="runtime-InstanceStatus"></a>

### InstanceStatus



| Field | Type | Label | Description |
| ----- | ---- | ----- | ----------- |
| meta | [meta.AuraeMeta](#meta-AuraeMeta) |  |  |
| status | [meta.Status](#meta-Status) |  |  |





 

 

 


<a name="runtime-Runtime"></a>

### Runtime
Runtime is a synchronous and immediate subsystem.
 Use the Runtime subsystem to start and stop executables, containers, and instances.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Exec | [Executable](#runtime-Executable) | [ExecutableStatus](#runtime-ExecutableStatus) |  |

 



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

