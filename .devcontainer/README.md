# Developing using a Dev Container

The image used for the container (Dockerfile.devcontainer) is meant to be usable as a local devevelopment environment on x86_64 and aarch64 systems. Visual Studio Code and Docker are [requirements](https://code.visualstudio.com/docs/devcontainers/containers#_system-requirements).

To get started using Dev Containers in Visual Studio Code follow the [tutorial](https://code.visualstudio.com/docs/devcontainers/tutorial).  More guidance is also available in the article [Developing inside a Container](https://code.visualstudio.com/docs/devcontainers/containers). 

## Notes

- Inside the container `make auraed-start` does not work since auraed is expected to be installed at `/usr/local/.cargo/bin/auraed` but is actually at `/usr/local/cargo/bin/auraed`. After installation (e.g., `make auraed`), the daemon can be launched with `auared` as it should be found using `$PATH`.
- Inside the container `make docs-serve` does not work as it depends on launching a docker container.