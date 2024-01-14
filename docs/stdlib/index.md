# The Aurae Standard Library

The Aurae Standard Library (stdlib or "the library") is a set of remote functions grouped together into logical groups called subsystems.

The library leverages [protobuf](https://github.com/protocolbuffers/protobuf) as the source of truth for the types, names, and function signatures for the library.

### What is a subsystem?

A subsystem is a smaller and scoped subsection of the library composed of RPCs and services. Subsystems are similar to "packages" or "modules" in programming languages such as [Rust](https://github.com/rust-lang/rust/tree/master/library/core/src). Kubernetes has API groups, and Linux itself has subsystems.

Each subsystem is unique. Each subsystem is liable to come with its own guarantees, and expectations.

In protobuf terms a subsystem is a group of remote procedure calls (RPCs) and [services](https://protobuf.dev/reference/protobuf/proto3-spec/#service_definition).

### What are resources?

Aurae is built on the concept of core resources that represent the main components of the system. Resources are like objects.

For example, Aurae has the concept of an `Executable` resource which represents an executable workload similar to systemd's [Unit](https://www.freedesktop.org/software/systemd/man/systemd.unit.html).

The core resources are intended to be fundamental and composable, similar to the objects and structures found in modern programming languages.

Resources are defined directly in the corresponding protobuf definition and later generated into code for various languages. A resource's corresponding message should never be passed directly to, or received directly from an RPC.

In protobuf terms a resource is a [message](https://protobuf.dev/programming-guides/proto3/#simple).

### What are services?

Services are a section of the API designed to be a way of grouping functionality together such that it can be enabled/disabled with authorization mechanisms.

A service should be discreet in the terms of how it mutates the system. For example if a service starts, it should stop. If a service allocates, it should free. And so on.

Services should be named after a resource or set of functionality around common resources.
Services should follow the `service NameService` paradigm as defined in the [style guide](https://protobuf.dev/programming-guides/style/).

For example the service that mutates a `Cell` should be called `CellService`.

### What are functions?

A function is a discreet piece of functionality designed to execute on the "backend", or directly by an Aurae Daemon server.

The library is designed to be executed procedurally and quickly. Many function calls per second is a reasonable expectation for any client.

In protobuf terms a function is a [remote procedure call (RPC)](https://protobuf.dev/programming-guides/proto3/#services)

### API Definition Convention

Generally follow [this style guide](https://protobuf.dev/programming-guides/style/) in the `.proto` files.

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
- The zero value enum variants should have the suffix `_UNSPECIFIED`
- Enums should NOT be nested, and their variants should be prefixed with the enum's name

```proto
enum FooBar {
  FOO_BAR_UNSPECIFIED = 0;
  FOO_BAR_FIRST_VALUE = 1;
  FOO_BAR_SECOND_VALUE = 2;
}
``` 

A notable exception to the public specification above is the Aurae project's preference for standardizing the objects that are used as the request and response messages.

The traditional convention that is meant to reduce the likelihood of future breaking changes and ease the creation of macros for generating code:

- RPC methods (e.g., `StartWidget`) should have dedicated request and response messages named `StartWidgetResponse` and `StopWidgetResponse`
- Objects (e.g., `Widget`) should be embedded directly into their corresponding `StartWidgetRequest`, `StopWidgetRequest`, etc style methods.

When deciding how to represent something in the API where there is an existing lower-level API, for example in the case of [cgroups](https://docs.kernel.org/admin-guide/cgroup-v2.html), prefer to represent the fields as close to the lower-level API as possible.  While it may be more natural to represent a list of CPUs as a repeated int, we prefer to offer familiarity to those users who know the lower-level API.
