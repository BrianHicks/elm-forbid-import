{ sources ? import ./nix/sources.nix { }, pkgs ? import sources.nixpkgs { }, ...
}:
let
  naersk = pkgs.callPackage sources.naersk { };
  gitignore = import sources.gitignore { };

  source = gitignore.gitignoreSource ./.;

  # naerk's checkPhase does not have the tests directory available in checkPhase,
  # so we work around it with a separate derivation. There's probably a nicer
  # way around this but I don't know it right now.
  package = naersk.buildPackage source;
in pkgs.stdenv.mkDerivation {
  name = package.name;
  version = package.version;
  src = source;

  buildInputs = [ package pkgs.jq ];
  buildPhase = "true";

  doCheck = true;
  checkPhase = ''
    env PATH="${package}/bin:$PATH" script/run-integration-tests.sh
  '';

  installPhase = ''
    ln -s ${package} $out
  '';
}

