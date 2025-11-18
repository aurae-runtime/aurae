# Aurae Community

Welcome to The Aurae Runtime project.

The project name is pronounced like the English word "aura" and is named after a minor Greek/Roman mythological deity, whose name means "breeze".

Aurae is a [Nivenly Foundation](https://github.com/nivenly) project and agrees to abide by the Nivenly covenant.

# Getting Involved

If you would like to get involved with Aurae development:

- Join our [discord](https://discord.gg/aTe2Rjg5rq).
- Read the [Nivenly Covenant](https://nivenly.org/covenant) which calls out the code of conduct.
- Read the [Contribution Guidelines](https://github.com/aurae-runtime/community/blob/main/CONTRIBUTING.md).
- Sign the [CLA](https://cla.nivenly.org/) to begin contributing as an individual contributor.

## Development workflow notes

- AuraeCI and the Makefile expect the `buf` CLI at version **1.60.0**. You can confirm locally with `buf --version`, and `hack/install-build-deps.sh` will install/upgrade to the pinned version if needed.
- The automatic license-header tooling (`make fmt` -> `hack/headers-write`) intentionally skips the `target/` and `vendor/` trees so Cargo caches are not mutated between builds. Please keep it that way when modifying helper scripts.

# What is Aurae?

[Aurae](https://github.com/aurae-runtime/aurae) is an opinionated turing complete scripting language built for the enterprise. Think of it like TypeScript for infrastructure platforms.

[Auraed](https://github.com/aurae-runtime/auraed) is the project core runtime daemon, auth and identity management system, and gRPC server that listens on a Unix domain socket.
