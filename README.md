# Aurae

Aurae is an opinionated turing complete scripting language built for the enterprise. Think of it like TypeScript for infrastructure platforms.

Aurae executes functions over gRPC against a daemon called `auraed` which is memory safe runtime of the project. The daemons can be networked together to form a mesh. 

Application teams and Infrastructure teams use the name language and standard library to manage their daily tasks.

```TypeScript
#!/usr/bin/env aurae

let helloContainer = container();
helloContainer.image("busybox");


let helloPod = pod();
helloPod.env("key", "value");
helloPod.env("foo", "bar");
helloPod.expose(80);
helloPod.expose(8080);

helloPod.add(helloContainer);

let aurae = connect();
let runtime = aurae.runtime();
runtime.run(helloPod);

```

### The Aurae Standard Library 

The ASL or Aurae Standard Library is composed of pillars of functionality called **subsystems**.
Aurae Subsystems resemble Linux subsystems and Kubernetes resource types.
 
 - Runtime
 - Schedule
 - Observe
 - Secret
 - Route
 - Mount
 
