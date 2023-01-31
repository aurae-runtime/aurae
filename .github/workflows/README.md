# GitHub Workflows

Please use the following naming convention such that we can easily map failed builds back to YAML files here in the repository.

```
$number-$friendlyname-$environment-$testcommands
```

Where **number** is just a unique identifier to easily map failed builds to YAML files. See below for more guidance on choosing a number.
Where **friendlyname** is a good descriptor to describe what is being checked (avoid words like "build" or "main" as they are too generic)
Where **environment** describes the environment it is running in, such as `alpine:latest` or `armv7`.
Where **testcommands** are the commands that a user can replicate on their computer! Do NOT test commands that can not be easily replicated!

```
007-linter-alpine-make-lint.yaml
```

A new linter running in alpine that tests the command `make lint` and **007** can easily be mapped backed to the file.

## Choosing a number
- 001-099: Reserved for workflows that don't use a container image
- 101-199: Reserved for workflows using the Dockerfile.build image
- 201-299: Reserved for workflows using the Dockerfile.test image

## Testing workflows

Workflows can be testing using the tool [act](https://github.com/nektos/act). You'll need to install that locally, then you can run commands to test each file individually, locally:

`make test-workflow file=001-cargo-install-ubuntu-make-install.yml`
