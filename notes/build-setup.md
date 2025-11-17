## Toolchain Notes

- Rust targets: `rust-toolchain.toml` pins stable 1.91.1 with `x86_64-unknown-linux-musl` as the required target (`rust-toolchain.toml:15-18`). On an x86_64 host run `rustup target add x86_64-unknown-linux-musl`; on aarch64 hosts override/add `aarch64-unknown-linux-musl` so `make` (which uses `$(uname -m)-unknown-linux-musl`) can build (`Makefile:224`). If you need to test older toolchains, use `rustup override set <version>` locally instead of editing `rust-toolchain.toml`.

- Buf CLI: Buf drives all protobuf/TypeScript code generation. Install Buf v1.50.0 (per the Makefile) and ensure `buf lint api` and `buf generate -v api` succeed before building.

- System toolchain bits used by dependencies:
  - `clang`/`libclang` (bindgen requirements from `virtio-bindings`)
  - `llvm` tooling (for eBPF compilation)
  - `python3` + `ninja` (rusty_v8 build when compiling `auraescript`)
  - musl-capable cross-compilers (`aarch64-linux-musl-gcc`, `x86_64-linux-musl-gcc` or equivalent). On Fedora you can install `musl-gcc`/`musl-clang` (symlink `/usr/bin/musl-gcc` to `aarch64-linux-musl-gcc`) or use the distro cross GCCs (`gcc-aarch64-linux-gnu`, `gcc-x86_64-linux-gnu`) and wrap/symlink them to the expected `*-linux-musl-gcc` names.
  - A portable option is to download musl.cc toolchains:

    ```bash
    mkdir -p ~/toolchains
    cd ~/toolchains
    curl -LO https://musl.cc/aarch64-linux-musl-cross.tgz
    tar xf aarch64-linux-musl-cross.tgz
    ```

    Then add the extracted `bin/` directory to your PATH (or symlink the `*-linux-musl-gcc` binaries into `~/.local/bin`).
