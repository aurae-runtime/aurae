# Aurae

Aurae is a turing complete scripting language and shell built for the enterprise. Think of it like TypeScript for infrastructure platforms.

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
 
