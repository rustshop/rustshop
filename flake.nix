{
  description = "RustShop - a fake shop that you can fork";

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

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, naersk, nixpkgs, flake-utils, flake-compat, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = pkgs.lib;
        fenix-pkgs = fenix.packages.${system};
        fenix-channel = fenix-pkgs.complete;

        craneLib = (crane.mkLib pkgs).overrideScope' (final: prev: {
          cargo = fenix-channel.cargo;
          rustc = fenix-channel.rustc;
        });

        commonArgs = {
          src = ./services;
          buildInputs = [
          ];
          nativeBuildInputs = [
            pkgs.pkgconfig
            fenix-channel.rustc
          ];
        };

        # filter source code at path `src` to include only the list of `modules`
        filterModules = modules: src:
          let
            basePath = toString src + "/";
          in
          lib.cleanSourceWith {
            filter = (path: type:
              let
                relPath = lib.removePrefix basePath (toString path);
                includePath =
                  (type == "directory" && builtins.match "^[^/]+$" relPath != null) ||
                  lib.any
                    (re: builtins.match re relPath != null)
                    ([ "Cargo.lock" "Cargo.toml" ".*/Cargo.toml" ] ++ builtins.concatLists (map (name: [ name "${name}/.*" ]) modules));
              in
              # uncomment to debug:
                # builtins.trace "${relPath}: ${lib.boolToString includePath}"
              includePath
            );
            inherit src;
          };

        workspaceDeps = craneLib.buildDepsOnly (commonArgs // {
          pname = "services-workspace-deps";
        });

        # a function to define both package and container build for a given service binary
        serviceApp = name: rec {
          package = craneLib.buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps;
            pname = name;

            src = filterModules [ "common-app" name ] ./services;

            cargoExtraArgs = "--bin ${name}";
          });

          container = pkgs.dockerTools.buildLayeredImage {
            name = name;
            contents = [ package ];
            config = {
              Cmd = [
                "${package}/bin/${name}"
              ];
              ExposedPorts = {
                "8000/tcp" = { };
              };
            };
          };
        };

        resGen = shortName:
          let
            name = "${shortName}-res-gen";
          in
          craneLib.buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps;
            pname = name;

            src = filterModules [ "common-res-gen" "common-app" name ] ./services;

            cargoExtraArgs = "--bin ${name}";
          });

        apps = {
          starter = serviceApp "starter";
          shopkeeper = serviceApp "shopkeeper";
        };

        resGens = {
          starter = resGen "starter";
          shopkeeper = resGen "shopkeeper";
        };

      in
      {
        packages = {
          default = apps.starter.package;

          app = {
            starter = apps.starter.package;
            shopkeeper = apps.shopkeeper.package;
          };

          res = {
            starter = resGens.starter;
            shopkeeper = resGens.shopkeeper;
          };

          cont = {
            starter = apps.starter.container;
            shopkeeper = apps.shopkeeper.container;
          };
        };

        devShells = {
          default =
            let
              # external project would import `rustshop` as a flake,
              # but we cheat, at least for now; but it is an external
              # components, and working on it requires `cd rustshop; nix develop`
              # as it maintains it's own flake
              rustshop = (import ./rustshop/default-system.nix) system;
            in
            pkgs.mkShell {
              buildInputs = workspaceDeps.buildInputs;
              nativeBuildInputs = workspaceDeps.nativeBuildInputs ++
                lib.attrsets.attrValues rustshop.packages."${system}" ++ [

                # extra binaries here
                fenix-pkgs.rust-analyzer
                fenix-channel.rustc
                fenix-channel.cargo

                # Lints
                # Note: we're using nixpkgs's `rustfmt` to avoid pulling in whole
                # `fenix-channel` into CI
                pkgs.rustfmt
                pkgs.terraform-ls
                pkgs.rnix-lsp
                pkgs.nodePackages.bash-language-server

                # Nix
                pkgs.nixpkgs-fmt
                pkgs.shellcheck

                # Utils
                pkgs.git
                pkgs.gh
              ];

              RUST_SRC_PATH = "${fenix-channel.rust-src}/lib/rustlib/src/rust/library";
              shellHook = ''
                for hook in misc/git-hooks/* ; do ln -sf "../../$hook" "./.git/hooks/" ; done
                ${pkgs.git}/bin/git config commit.template misc/git-hooks/commit-template.txt
                . ${rustshop.default}/usr/share/rustshop/shell-hook.sh
              '';
            };

          # this shell is used only in CI, so it should contain minimum amount
          # of stuff to avoid building and caching things we don't need
          ci = pkgs.mkShell {
            nativeBuildInputs = [
              pkgs.rustfmt
              pkgs.nixpkgs-fmt
              pkgs.shellcheck
              pkgs.git
            ];
          };
        };
      });
}
