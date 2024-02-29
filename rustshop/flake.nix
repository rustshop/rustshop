{
  description = "rustshop infra utilities";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";

    # this is needed for systems default-system.nix
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    flakebox = {
      url = "github:dpc/flakebox?rev=49117df15209701f3e13ba2bcf514b550955e7b4";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

        projectName = "rustshop";

        flakeboxLib = flakebox.lib.${system} {
          config = {
            github.ci.cachixRepo = "rustshop";
            toolchain.channel.default = "complete";
            github.ci.buildOutputs = [ ".#ci.${projectName}" ];
          };
        };

        buildPaths = [
          "Cargo.toml"
          "Cargo.lock"
          "bin"
          "env"
          "templates"
          "shell-hook.sh"
        ];

        buildSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = projectName;
            path = ./.;
          };
          paths = buildPaths;
        };

        multiBuild =
          (flakeboxLib.craneMultiBuild { }) (craneLib':
            let
              craneLib = (craneLib'.overrideArgs ({
                pname = projectName;
                src = buildSrc;
                nativeBuildInputs = [ ];
              } // craneLib'.crateNameFromCargoToml { cargoToml = ./bin/Cargo.toml; }));
            in
            {
              ${projectName} = craneLib.buildPackage {

                postInstall = ''
                  mkdir -p "$out/usr/share/rustshop"
                  cp ./shell-hook.sh $out/usr/share/rustshop
                '';
              };
            });



        rustshop = multiBuild.rustshop;
        wrapBins = { pkgs ? pkgs }: {

          # wrap to auto inject account envs: terraform
          terraform = (pkgs.writeShellScriptBin "terraform" "exec ${rustshop}/bin/rustshop wrap ${pkgs.terraform}/bin/terraform \"$@\"");

          # wrap to auto inject account envs: aws
          aws = (pkgs.writeShellScriptBin "aws" "exec ${rustshop}/bin/rustshop wrap ${pkgs.awscli2}/bin/aws \"$@\"");

          # wrap to auto inject account envs: aws
          kops = (pkgs.writeShellScriptBin "kops" "exec ${rustshop}/bin/rustshop wrap ${pkgs.kops}/bin/kops \"$@\"");

          # wrap to auto inject account envs: aws
          kubectl = (pkgs.writeShellScriptBin "kubectl" "exec ${rustshop}/bin/rustshop wrap ${pkgs.kubectl}/bin/kubectl \"$@\"");

          # wrap to auto inject account envs: aws
          helm = (pkgs.writeShellScriptBin "helm" "exec ${rustshop}/bin/rustshop wrap ${pkgs.kubernetes-helm}/bin/helm \"$@\"");

          # alias `kubectl` to just `kc`
          kc = (pkgs.writeShellScriptBin "kc" "exec ${rustshop}/bin/rustshop wrap ${pkgs.kubectl}/bin/kubectl \"$@\"");

          # alias `rustshop` to just `shop`
          shop = (pkgs.writeShellScriptBin "shop" "exec ${rustshop}/bin/rustshop  \"$@\"");
        };
        wrappedBins = wrapBins {
          inherit pkgs;
        };
      in
      {
        lib = { inherit wrapBins; };
        legacyPackages = multiBuild;

        packages = {
          inherit rustshop;
          default = rustshop;
        } // wrappedBins;

        devShells = flakeboxLib.mkShells { };
      });
}
