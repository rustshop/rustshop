{
  description = "rustshop infra utilities";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = pkgs.lib;
        stdenv = pkgs.stdenv;
        fenix-pkgs = fenix.packages.${system};
        fenix-channel = fenix-pkgs.complete;

        craneLib = (crane.mkLib pkgs).overrideScope' (final: prev: {
          cargo = fenix-channel.cargo;
          rustc = fenix-channel.rustc;
        });

        commonArgs = {
          src = ./.;
          buildInputs = [
          ] ++ lib.optionals stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];
          nativeBuildInputs = [
            pkgs.pkgconfig
            fenix-channel.rustc
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "rustshop-deps";
          doCheck = false;
        });

        rustshopPkg = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "rustshop";

          cargoExtraArgs = "--bin rustshop";

          postInstall = ''
            mkdir -p "$out/usr/share/rustshop"
            cp ./shell-hook.sh $out/usr/share/rustshop
          '';
        });

      in
      {
        packages = rec {
          default = rustshop;

          rustshop = rustshopPkg;

          # alias `rustshop` to just `shop`
          shop = (pkgs.writeShellScriptBin "shop" "exec -a \"$0\" ${rustshop}/bin/rustshop  \"$@\"");

          # alias `kubectl` to just `kc`
          kc = (pkgs.writeShellScriptBin "kc" "exec -a \"$0\" ${kubectl}/bin/kubectl \"$@\"");

          # wrap to auto inject account envs: terraform
          terraform = (pkgs.writeShellScriptBin "terraform" "exec -a \"$0\" ${rustshop}/bin/rustshop wrap ${pkgs.terraform}/bin/terraform \"$@\"");

          # wrap to auto inject account envs: aws
          aws = (pkgs.writeShellScriptBin "aws" "exec -a \"$0\" ${rustshop}/bin/rustshop wrap ${pkgs.awscli2}/bin/aws \"$@\"");

          # wrap to auto inject account envs: aws
          kops = (pkgs.writeShellScriptBin "kops" "exec -a \"$0\" ${rustshop}/bin/rustshop wrap ${pkgs.kops}/bin/kops \"$@\"");

          # wrap to auto inject account envs: aws
          kubectl = (pkgs.writeShellScriptBin "kubectl" "exec -a \"$0\" ${rustshop}/bin/rustshop wrap ${pkgs.kubectl}/bin/kubectl \"$@\"");

          # wrap to auto inject account envs: aws
          helm = (pkgs.writeShellScriptBin "helm" "exec -a \"$0\" ${rustshop}/bin/rustshop wrap ${pkgs.kubernetes-helm}/bin/helm \"$@\"");
        };

        devShell = pkgs.mkShell {
          buildInputs = cargoArtifacts.buildInputs;
          nativeBuildInputs = cargoArtifacts.nativeBuildInputs ++ [
            fenix-pkgs.rust-analyzer
            fenix-channel.rustfmt
            fenix-channel.rustc
            fenix-channel.cargo

            pkgs.rnix-lsp
            pkgs.nodePackages.bash-language-server
          ];
          RUST_SRC_PATH = "${fenix-channel.rust-src}/lib/rustlib/src/rust/library";
        };
      });
}
