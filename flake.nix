{
  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };

    flake-utils.follows = "rust-overlay/flake-utils";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = inputs: with inputs; # pass through all inputs and bring them into scope
    # Build the output set for each default system and map system sets into
    # attributes, resulting in paths such as:
    # nix build .#packages.x86_64-linux.<name>
    flake-utils.lib.eachDefaultSystem (system:

      # let-in expressions, very similar to Rust's let bindings.  These names
      # are used to express the output but not themselves paths in the output.
      let
        # Create nixpkgs that contains the rust-overlay.
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in
      rec {
        # This is contents of the (recursive) output set, which is expressed
        # for each system.

        # The default development shell for Aurae, launched with `nix develop`.
        devShells.default = pkgs.mkShell {
          shellHook = ''
            # `make musl` requires a default toolchain.
            rustup default stable
          '';

          buildInputs = with pkgs; [
            buf
            libseccomp
            protobuf
            rustup
          ];
        };
      }
    );
}
