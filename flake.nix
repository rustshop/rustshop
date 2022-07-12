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
          # external project would import `rustshop` as a flake,
          # but we cheat, at least for now
          rustshop = (import ./rustshop/default-system.nix) system;
          services = (import ./services/default-system.nix) system;
          servicesNativeBuildInputs = services.outputs.devShell."${system}".nativeBuildInputs;
        in pkgs.mkShell {
          buildInputs =
            servicesNativeBuildInputs ++
            lib.attrsets.attrValues rustshop.packages."${system}" ++ [
              # extra binaries here
            ];

          shellHook = ''
            . ${rustshop.default}/usr/share/rustshop/shell-hook.sh
          '';
        };
      });
}
