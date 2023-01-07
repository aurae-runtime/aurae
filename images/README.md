# Images

This folder holds the container image manifests that are needed by the project. Changes to container manifests (currently Dockerfile's) that are built and stored in the Github Repository via the Github Actions pipeline cannot be merged in from a fork of the repository - the Github Token will not let their repository push to our package registry.

## Build image (Dockerfile.build)

Used by Github Actions (GHA) as a build container. It uses docker [buildx](https://github.com/docker/buildx), which enables multi-stage builds and [GHA native layer caching](https://docs.docker.com/build/cache/backends/gha/). The container is automatically built as the first step in the GHA pipeline.

This image can also be used locally as a way to run tests in the same environment as GHA uses, and without requiring a local rust installation.

1. Build the container locally. Make sure you are in the root `aurae` directory for these steps.

   `make oci-image-build tag=builder ocifile=./docker/Dockerfile.build flags='--target local'`

2. Pass in whichever `make` command you would like to run inside the container. This example runs `make docs` inside the build container.

   `make oci-make tag=builder command=docs`

Since the `auraed` directory is mounted inside this container, you can also pass commands like `fmt`. This will modify files in the project.

- `make oci-make tag=builder command=fmt`
