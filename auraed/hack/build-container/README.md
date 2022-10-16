aurae Build Container
---------------------

The Build Container is used to provide a build environment, that is consistent across different build hosts.
Currently the Build Container is derived from the official [Rust 1-bullseye Docker container](https://hub.docker.com/_/rust). It is used to build the Linux kernel, the auraed rust application and the initramfs.

All dynamically linked libraries, that will end up in the initramfs are sourced from this Build Container!
So if there's a vulnerability in the Build Container's libraries, it will also be in the final aurae initramfs!

You should care about keeping this container up to date!
Rebuild it from time to time using `make build-container`!