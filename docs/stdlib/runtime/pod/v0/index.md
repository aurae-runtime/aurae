# Protocol Documentation
<a name="top"></a>

## Table of Contents

- [pod.proto](#pod-proto)
    - [Container](#aurae-runtime-pod-v0-Container)
    - [Pod](#aurae-runtime-pod-v0-Pod)
    - [PodServiceAllocateRequest](#aurae-runtime-pod-v0-PodServiceAllocateRequest)
    - [PodServiceAllocateResponse](#aurae-runtime-pod-v0-PodServiceAllocateResponse)
    - [PodServiceFreeRequest](#aurae-runtime-pod-v0-PodServiceFreeRequest)
    - [PodServiceFreeResponse](#aurae-runtime-pod-v0-PodServiceFreeResponse)
    - [PodServiceStartRequest](#aurae-runtime-pod-v0-PodServiceStartRequest)
    - [PodServiceStartResponse](#aurae-runtime-pod-v0-PodServiceStartResponse)
    - [PodServiceStopRequest](#aurae-runtime-pod-v0-PodServiceStopRequest)
    - [PodServiceStopResponse](#aurae-runtime-pod-v0-PodServiceStopResponse)
  
    - [PodService](#aurae-runtime-pod-v0-PodService)
  
- [Scalar Value Types](#scalar-value-types)



<a name="pod-proto"></a>
<p align="right"><a href="#top">Top</a></p>

## pod.proto



<a name="aurae-runtime-pod-v0-Container"></a>

### Container







<a name="aurae-runtime-pod-v0-Pod"></a>

### Pod







<a name="aurae-runtime-pod-v0-PodServiceAllocateRequest"></a>

### PodServiceAllocateRequest







<a name="aurae-runtime-pod-v0-PodServiceAllocateResponse"></a>

### PodServiceAllocateResponse







<a name="aurae-runtime-pod-v0-PodServiceFreeRequest"></a>

### PodServiceFreeRequest







<a name="aurae-runtime-pod-v0-PodServiceFreeResponse"></a>

### PodServiceFreeResponse







<a name="aurae-runtime-pod-v0-PodServiceStartRequest"></a>

### PodServiceStartRequest







<a name="aurae-runtime-pod-v0-PodServiceStartResponse"></a>

### PodServiceStartResponse







<a name="aurae-runtime-pod-v0-PodServiceStopRequest"></a>

### PodServiceStopRequest







<a name="aurae-runtime-pod-v0-PodServiceStopResponse"></a>

### PodServiceStopResponse






 

 

 


<a name="aurae-runtime-pod-v0-PodService"></a>

### PodService
A pod is a higher level abstraction than Aurae cells, and to most users
/ will look at feel like one or more &#34;containers&#34;.
/
/ Pods will run an OCI compliant container image.
/
/ A pod is a group of one or more containers with shared network and storage.

| Method Name | Request Type | Response Type | Description |
| ----------- | ------------ | ------------- | ------------|
| Allocate | [PodServiceAllocateRequest](#aurae-runtime-pod-v0-PodServiceAllocateRequest) | [PodServiceAllocateResponse](#aurae-runtime-pod-v0-PodServiceAllocateResponse) |  |
| Start | [PodServiceStartRequest](#aurae-runtime-pod-v0-PodServiceStartRequest) | [PodServiceStartResponse](#aurae-runtime-pod-v0-PodServiceStartResponse) |  |
| Stop | [PodServiceStopRequest](#aurae-runtime-pod-v0-PodServiceStopRequest) | [PodServiceStopResponse](#aurae-runtime-pod-v0-PodServiceStopResponse) |  |
| Free | [PodServiceFreeRequest](#aurae-runtime-pod-v0-PodServiceFreeRequest) | [PodServiceFreeResponse](#aurae-runtime-pod-v0-PodServiceFreeResponse) |  |

 



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

