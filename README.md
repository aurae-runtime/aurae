# Aurae

Aurae is a distributed systems runtime written in Rust. This particular project is the turing complete shell that sits on top of everything.

#### What it is.

The `aurae` tool is a command line shell, that is processed by a backing daemon called `auraed`. The shell is interpreted at runtime and works like a regular old Linux style programming langauge.

```
#!/bin/aurae

print("----------------------");
print("Hello, welcome to Auare");
print("----------------------");


```

#### What it does.

Aurae gives application teams an extremely versitle set of tools, patterns, and primitives to interact with a core daemon called `auraed`. The daemon is built from problems encountered at running Kubernetes backed platforms at Microsoft, Google, VMware, Twilio and more.
