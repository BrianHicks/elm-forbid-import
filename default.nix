{ sources ? import ./nix/sources.nix { }, pkgs ? import sources.nixpkgs { }, ...
}:
let
  naersk = pkgs.callPackage sources.naersk { };
  gitignore = pkgs.callPackage sources.gitignore { };
in naersk.buildPackage (gitignore.gitignoreSource ./.)

