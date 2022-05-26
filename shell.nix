{ ... }:
let
  sources = import ./nix/sources.nix;
  nixpkgs = import sources.nixpkgs { };
  niv = import sources.niv { };
  naersk = nixpkgs.callPackage sources.naersk { };

  cargo-lichking = naersk.buildPackage sources.cargo-lichking;
in with nixpkgs;
stdenv.mkDerivation {
  name = "elm-forbid-import";
  buildInputs = [
    niv.niv
    git

    # Rust
    cargo
    cargo-lichking
    rustPackages.clippy
    rustPackages.rustfmt
    rustc

    # Benchmarking + Optimization
    hyperfine

    # Testing
    jq
  ];
}
