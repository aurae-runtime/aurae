# GitHub Workflows

Please use the following naming convention such that we can easily map failed builds back to YAML files here in the repository.

```
$number-$friendlyname-$environment-$testcommands
```

Where **number** is just a unique identifier to easily map failed builds to YAML files.
Where **friendlyname** is a good descriptor to describe what is being checked (avoid words like "build" or "main" as they are too generic)
Where **environment** describes the environment it is running in, such as `alpine:latest` or `armv7`.
Where **testcommands** are the commands that a user can replicate on their computer! Do NOT test commands that can not be easily replicated!

```
007-linter-alpine-make-lint.yaml
```
A new linter running in alpine that tests the command `make lint` and **007** can easily be mapped backed to the file.

