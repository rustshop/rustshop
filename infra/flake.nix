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
            in
            [
              pkgs.terraform
              pkgs.awscli2
              aws-bootstrap-pkgs.default
            ];

          shellHook = ''
            export PATH="$PWD/utils:$PATH"
            # Initialize variable files from templates
            if [ ! -e ".env" ]; then
              echo 'Creating .env' 1>&2
              cp .env.template .env
            fi
            if [ ! -e ".shrc" ]; then
              echo 'Creating .shrc' 1>&2
              cp .shrc.template .shrc
            fi
            set -a
            . ./.env
            set +a
            . ./.shrc
          '';
        };
      });
}
