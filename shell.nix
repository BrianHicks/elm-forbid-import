{ ... }:
let
  sources = import ./nix/sources.nix;
  nixpkgs = import sources.nixpkgs { };
  niv = import sources.niv { };
in with nixpkgs;
stdenv.mkDerivation {
  name = "elm-forbid-import";
  buildInputs = [
    niv.niv
    git

    # Rust
    cargo
    cargo-watch
    rustPackages.clippy
    rustPackages.rustfmt
    rustc
  ];
}
