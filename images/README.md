# Images

This folder holds the container image manifests that are needed by the project. Changes to container manifests (currently Dockerfile's) that are built and stored in the Github Repository via the Github Actions pipeline cannot be merged in from a fork of the repository - the Github Token will not let their repository push to our package registry.

## Build image (Dockerfile.build)

Used by Github Actions (GHA) as a build container. It uses docker [buildx](https://github.com/docker/buildx), which enables multi-stage builds and [GHA native layer caching](https://docs.docker.com/build/cache/backends/gha/). The container is automatically built as the first step in the GHA pipelines that may need it.

Please (seriously please) be careful about adding commands here. This is our core way of validating that our binary is "healthy". If we need to install anything with the word "lib" in it to get the build to pass, we likely should be having other discussions instead of adding commands here. For example we should NOT be adding libraries such as "libseccomp"
or "libdbus".

                  Do not add GNU libraries here!
        If in doubt, please ask in Discord in the build channel.

## Testing image (Dockerfile.test)

Used by Github Actions (GHA) as a testing container. It uses docker [buildx](https://github.com/docker/buildx), which enables multi-stage builds and [GHA native layer caching](https://docs.docker.com/build/cache/backends/gha/). The container is automatically built as the first step in the GHA pipelines that may need it.

This image can also be used locally as a way to run tests in the same environment as GHA uses, and without requiring a local rust installation.

1. Build the container locally. Make sure you are in the root `aurae` directory for these steps.

   `make oci-image-build tag=builder ocifile=./images/Dockerfile.test`

2. Pass in whichever `make` command you would like to run inside the container. This example runs `make docs` inside the build container.

   `make oci-make tag=builder command=make docs`

Since the `auraed` directory is mounted inside this container, you can also pass commands like `fmt`. This will modify files in the project.

- `make oci-make tag=builder command=make fmt`
