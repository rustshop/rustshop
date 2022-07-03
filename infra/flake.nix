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
              utils = (import ./_rustshop/default-system.nix) system;
              rustshop-terraform = utils.packages."${system}".rustshop-terraform;
              rustshop-aws = utils.packages."${system}".rustshop-aws;
            in
            [
              # pkgs.kops
              # pkgs.kubectl
              utils.packages."${system}".aws-bootstrap
              # wrap these binaries:
              (pkgs.writeShellScriptBin "terraform" "exec -a \"$0\" ${rustshop-terraform}/bin/rustshop-terraform ${pkgs.terraform}/bin/terraform \"$@\"")
              (pkgs.writeShellScriptBin "aws" "exec -a \"$0\" ${rustshop-aws}/bin/rustshop-aws ${pkgs.awscli2}/bin/aws \"$@\"")
            ];

          shellHook = ''
            export RUSTSHOP_ROOT="`pwd`"
            if [ ! -e ".env" ]; then
              echo 'Creating .env' 1>&2
              cp ${./_rustshop/templates/env.template} ".env"
              chmod 0600 .env
            fi

            . ${./_rustshop/shell-hook.sh}
          '';
        };
      });
}
