{
  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs; # pass through all inputs and bring them into scope

    # Build the output set for each default system and map system sets into
    # attributes, resulting in paths such as:
    # nix build .#packages.x86_64-linux.<name>
    flake-utils.lib.eachDefaultSystem (system:

      # let-in expressions, very similar to Rust's let bindings.  These names
      # are used to express the output but not themselves paths in the output.
      let

        # create nixpkgs that contains rustBuilder from cargo2nix overlay
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default rust-overlay.overlays.default ];
        };

        patchedSource = pkgs.runCommand "patch-source" {} ''
          set +x
          # Copy the api folder into the two projects
          mkdir -p $out
          cp -r ${builtins.toString ./.}/* $out/
          chmod -R +w $out
          cd $out
          ls -al
          cp -r api auraed
          cp -r api auraescript

          # Replace '../api' with 'api' in both scripts
          sed -i 's/\.\.\/api/api/g' auraed/build.rs
          sed -i 's/\.\.\/api/api/g' auraescript/build.rs
        '';

        # create the workspace & dependencies package set
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.64.0";
          packageFun = import ./Cargo.nix;
          workspaceSrc = patchedSource;
          packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++  [
            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "libseccomp";
              overrideAttrs = drv: {
              propagatedNativeBuildInputs = drv.propagatedNativeBuildInputs or [ ] ++ [
                pkgs.libseccomp.dev
              ];
            };
          })
          ];
        };

      in rec {
        # this is the output (recursive) set (expressed for each system)

        # the packages in `nix build .#packages.<system>.<name>`
        packages = {
          # nix build .#hello-world
          # nix build .#packages.x86_64-linux.hello-world
          auraed = (rustPkgs.workspace.auraed {}).bin;
          auraescript = (rustPkgs.workspace.auraescript {}).bin;
          # nix build
          default = packages.auraed; # rec
        };

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
