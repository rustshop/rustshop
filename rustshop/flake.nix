{
  description = "rustshop infra utilities";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    flakebox = {
      url = "github:rustshop/flakebox?rev=7a179ea785b1e6109ad37e15f8639d0f057f1d84";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flakebox }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        projectName = "rustshop";

        flakeboxLib = flakebox.lib.${system} {
          config = {
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

        packages = {
          inherit rustshop;
          default = rustshop;
        } // wrappedBins;

        devShells = flakeboxLib.mkShells { };
      });
}
