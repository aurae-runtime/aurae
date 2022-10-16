# Aurae Quickstart

Now that you have [built Aurae from source](/build) you can begin using Aurae.

### Running the Daemon 

Aurae will run on any system, even if `systemd` or another init daemon is currently active. 

```bash 
sudo -E auraed
```

### Writing your first AuraeScript 

First create an executable script anywhere you like.

```bash
touch ~/hello.aurae
chmod +x ~/.hello.aurae 
```

Next add the following content. 

```typescript
let aurae = connect();
let runtime = aurae.runtime();
aurae.info().json();

let example = exec("echo 'Hello World!'");
runtime.start(example).json();
```

You can now run your first AuraeScript.

```bash 
~/hello.aurae
```

Your output should be in valid JSON which should look similar to the following:

```json 
{
  "subject_common_name": "nova.unsafe.aurae.io",
  "issuer_common_name": "unsafe.aurae.io",
  "sha256_fingerprint": "SHA256:7afa7cbf54dacf8368fd7407039594264c5bb22eaa7f8de5017af53f5ab240b0",
  "key_algorithm": "RSA"
}
{
  "meta": {
    "name": "echo 'Hello World!'",
    "message": "-"
  },
  "proc": {
    "pid": 1428
  },
  "status": 6,
  "stdout": "'Hello World!'\n",
  "stderr": "",
  "exit_code": "exit status: 0"
}
```

As long as the `.json()` method is used for output, aurae scripts can be piped to `jq` for easy usage.

```bash 
~/hello.aurae | jq -r .stdout
```