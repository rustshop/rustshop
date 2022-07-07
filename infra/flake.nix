{
  description = "Infra";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = nixpkgs.lib;
      in
      {
        devShells.default =  let
          rustshop = (import ./_rustshop/default-system.nix) system;
        in pkgs.mkShell {
          buildInputs =
            lib.attrsets.attrValues rustshop.packages."${system}" ++ [
              # extra binaries here
            ];

          shellHook = ''
            . ${rustshop.default}/usr/share/shell-hook.sh
          '';
        };
      });
}
