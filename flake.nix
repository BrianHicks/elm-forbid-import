{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };

        naersk = pkgs.callPackage inputs.naersk { };
      in
      {
        defaultPackage = naersk.buildPackage {
          src = ./.;
        };

        formatter = pkgs.nixpkgs-fmt;

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            git

            # Rust
            cargo
            rustPackages.clippy
            rustPackages.rustfmt
            rustc

            # Benchmarking + Optimization
            hyperfine

            # Testing
            jq
          ];
        };
      }
    );
}
