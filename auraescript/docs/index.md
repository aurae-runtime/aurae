# AuraeScript

 - Runtime
    - Executing Commands 


### Connecting to an `auraed` socket. 

By default AuraeScript will look for a `config` file that matches the following syntax or the [default.config.toml](https://github.com/aurae-runtime/auraescript/blob/main/default.config.toml). 

```toml
# Client Cert Material
[auth]
ca_crt = "~/.aurae/pki/ca.crt"
client_crt = "~/.aurae/pki/_signed.client.nova.crt"
client_key = "~/.aurae/pki/client.nova.key"

# System Configuration
[system]
socket = "/var/run/aurae/aurae.sock"
```

In order of priority the following locations will be checked by default for a `config` file.

 - ${HOME}/.aura/config
 - /etc/aurae/config
 - /var/lib/aurae/config

After your filesystem is set up with valid mTLS material you can run the following script to validate you are authenticating with the system.

```TypeScript
#!/usr/bin/env auraescript
// info.aurae

let aurae = connect();
aurae.info().json();
```

Which can be ran directly with your normal user privileges. 

```bash
chmod +x info.aurae
./info.aurae
```

All of the output with Aurae can be displayed as valid JSON.

```json
{
  "subject_common_name": "nova.unsafe.aurae.io",
  "issuer_common_name": "unsafe.aurae.io",
  "sha256_fingerprint": "SHA256:7afa7cbf54dacf8368fd7407039594264c5bb22eaa7f8de5017af53f5ab240b0",
  "key_algorithm": "RSA"
}
```


