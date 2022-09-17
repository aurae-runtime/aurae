# Aurae

Aurae is a Turing complete platform infrastructure language.

### Compile and Install

To compile `aurae` by itself check out this repository and use the Makefile.

```bash
make # Will compile and install Aurae using Cargo.
```

Or manually using Cargo. 

```bash
cargo build 
cargo install --path .
```

### Connecting to Auraed

After `aurae` is compiled and installed to your local `$PATH` you can begin writing scripts to interface with [auraed](https://github.com/aurae-runtime/auraed).

First generate self-signed TLS certificates for the local daemon and your client to use.

```bash 
sudo -E make pki config # Generate TLS material and install to /etc and your $HOME directory
```

You should now see secret material installed in two locations.

 - /etc/aurae/pki
 - $HOME/.aurae/pki

And a TOML config file located in

 - $HOME/.aurae/config

Start and run [auraed](https://github.com/aurae-runtime/auraed) and you can begin writing scripts.

```typescript
#!/usr/bin/env aurae
let aurae = connect();
aurae.info();
```
### Architecture 

See the [whitepaper](https://docs.google.com/document/d/1dA591eipsgWeAlaSwbYNQtAQaES243IIqXPAfKhJSjU/edit#heading=h.vknhjb3d4yfc).

