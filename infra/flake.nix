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
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs =
            let
              aws-bootstrap-pkgs = (import ./utils/aws-bootstrap/default-system.nix) system;
              utils = (import ./utils/default-system.nix) system;
            in
            [
              pkgs.terraform
              pkgs.awscli2
              pkgs.kops
              pkgs.kubectl
              utils.packages."${system}".terraform-wrapper
              utils.packages."${system}".terraform-wrapper
            ];

          shellHook = ''
            . ${./utils/shell-hook.sh}
          '';
        };
      });
}
