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
              aws-bootstrap = utils.packages."${system}".aws-bootstrap;
              rustshop-terraform = utils.packages."${system}".rustshop-terraform;
              rustshop-aws = utils.packages."${system}".rustshop-aws;
            in
            [
              # pkgs.kops
              # pkgs.kubectl

              # wrap terraform to auto inject account envs
              (pkgs.writeShellScriptBin "terraform" "exec -a \"$0\" ${rustshop-terraform}/bin/rustshop-terraform ${pkgs.terraform}/bin/terraform \"$@\"")
              # wrap aws to auto inject account envs
              (pkgs.writeShellScriptBin "aws" "exec -a \"$0\" ${rustshop-aws}/bin/rustshop-aws ${pkgs.awscli2}/bin/aws \"$@\"")
              # aws-bootstrap is supposed to work without account env injections, but uses `aws` underneath so disable account env injection
              # Note: `exec -a ... env ...` doesn't work. `env` doesn't like it, because it uses it when call via hashbang.
              (pkgs.writeShellScriptBin "aws-bootstrap" "exec env RUSTSHOP_NO_WRAP=true ${aws-bootstrap}/bin/aws-bootstrap \"$@\"")
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
