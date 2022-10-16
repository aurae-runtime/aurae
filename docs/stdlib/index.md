# The Aurae Standard Library

The Aurae Standard Library (stdlib or "the library") is a set of remote functions grouped together into logical groups called subsystems.

The library leverages [protobuf](https://github.com/protocolbuffers/protobuf) as the source of truth for the types, names, and function signatures for the library.

### What is a subsystem? 

A subsystem is a smaller and scoped subsection of the library. Subsystems are similar to "packages" or "modules" in programming languages such as [Rust](https://github.com/rust-lang/rust/tree/master/library/core/src). Kubernetes as API groups, and Linux itself has subsystems. 

Each subsystem is unique. Each subsystem is liable to come with its own guarantees, and expectations.
For example the runtime subsystem is adamantly a synchronous subsystem which creates an imperative experience for the client. 
Contrarywise, the schedule subsystem is adamantly an asynchronous subsystem which instills a declarative model for the client.

In protobuf terms a subsystem is a [service](https://developers.google.com/protocol-buffers/docs/proto3#services).

### What are objects?

Aurae is built on the concept of core objects that are useful to distributed systems engineers. 
For example, Aurae has the concept of an `Executable` object which can be passed to `runtime.StartExecutable` and `runtime.StopExecutable` functions respectively. 

The core objects are intended to be fundamental and composable, similar to the objects and structures found in modern programming languages.

Objects are defined directly in the corresponding protobuf definition and later generated into code for various languages.

In protobuf terms an object is a [message](https://developers.google.com/protocol-buffers/docs/proto3#simple).

### What are functions?

A function is a discreet piece of functionality designed to execute on the "backend", or directly by an Aurae Daemon server.

The library is designed to be executed procedurally and quickly. Many function calls per second is a reasonable expectation for any client.

In protobuf terms a function is a [remote procedure call (RPC)](https://developers.google.com/protocol-buffers/docs/proto3#services)

### What about metadata? 

Similar to Kubernetes, Aurae defines some common objects which are embedded in some or all objects in the standard library.

Every Aurae object must embed `meta.AuraeMeta` implying that every object in the library will have a `.name` and a `.message` field.

```proto 
message AuraeMeta {
  string name = 1;
  string message = 2;
}
```

There are other common objects such as `meta.ProcessMeta` which is embedded in any object that has a concept of an executing runtime process.

### API Definition Convention

Generally follow [this style guide](https://developers.google.com/protocol-buffers/docs/style) in the proto files.

It is short, but the main points are:

- Files should be named `lower_snake_case.proto`
- Files should be ordered in the following manner

```proto
// AURAE LICENSE HEADER

syntax = "proto3";

package lower_snake_case_package_name;

// imports sorted alphabetically
import "path/to/dependency.proto";
import "path/to/other.proto";

// file options

// everything else

``` 

Generally follow these rules:

- Services should be named `UpperCamelCase` (aka PascalCase)
- Service methods should be named `UpperCamelCase`
- Messages should be named `UpperCamelCase`
- Field names, including `oneof` and extension names, should be `snake_case`
- `repeated` fields should have pluralized names
- Enums should be named `UpperCamelCase`
- Enum variants should be `SCREAMING_SNAKE_CASE`
- (Suggested) The zero value enum variants should have the suffix `UNSPECIFIED`
- (Suggested) Enums should NOT be nested, and their variants should be prefixed with the enum's name

```proto
enum FooBar {
  FOO_BAR_UNSPECIFIED = 0;
  FOO_BAR_FIRST_VALUE = 1;
  FOO_BAR_SECOND_VALUE = 2;
}
``` 

A notable exception to the public specification above is the Aurae projects preference for standardizing the objects that are used as the request and response messages.

The traditional convention that is meant to reduce the likelihood of future breaking changes and ease the creation of macros for generating code:

- rpc methods (e.g., `StartWidget`) should have dedicated request and response messages named `StartWidgetResponse` and `StartWidgetResponse`
- objects (e.g., `Widget`) should be embedded directly into their corresponding `StartWidgetRequest`, `StopWidgetReqyest`, etc style methods.
