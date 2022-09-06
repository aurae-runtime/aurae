# Aurae

Aurae is a turing complete scripting language for platform and application teams. Think of it like TypeScript for platforms similar to Kubernetes.

Use this executable as a runtime alternative to YAML.

### Run a Pod with Aurae

```TypeScript

let aurae = connect();
let runtime = aurae.runtime();

mypod = pod("nginx");
mypod.label("key", "value");
mypod.expose(8080);
mypod.expose(8081);
mypod.env("USERNAME", "nova");
mypod.env("PASSWORD", aurae.secret("nova_password"));
mypod.env("VERBOSE", false);

runtime.run(mypod);

```
