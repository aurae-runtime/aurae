# Development Environments

In an effort to ease the process of getting started and contributing to the project, the community documents development environments that are expected to work. **Please note, x86_64 architecture is currently the only officially supported target for running the Aurae Daemon (auared).**

1. [Apple Silicon - CLion & Parallels Desktop](macOS/apple-silicon/clion-parallels.md)
    - Uses CLion and an Ubuntu VM via Parallels Desktop to provide a development environment with a GUI.
2. [macOS - Lima](macOS/lima.md)
    - Uses an Ubuntu VM via [Lima](https://lima-vm.io/) to provide a development environment with an optional mounted
      unix socket for interacting with auread on the host machine.