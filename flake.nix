{
  description = "RustShop - a fake shop that you can fork";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane?ref=v0.5.1";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
        lib = pkgs.lib;

        fenix-toolchain = (fenix.packages.${system}.complete.withComponents [
          "rustc"
          "cargo"
          "clippy"
          "rust-analysis"
          "rust-src"
          "rustfmt"
        ]);

        fenix-channel = fenix.packages.${system}.complete;

        craneLib = crane.lib.${system}.overrideToolchain fenix-toolchain;

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

        # Filter only files needed to build project dependencies
        #
        # To get good build times it's vitally important to not have to
        # rebuild derivation needlessly. The way Nix caches things
        # is very simple: if any input file changed, derivation needs to
        # be rebuild.
        #
        # For this reason this filter function strips the `src` from
        # any files that are not relevant to the build.
        #
        # Lile `filterWorkspaceFiles` but doesn't even need *.rs files
        # (because they are not used for building dependencies)
        filterWorkspaceDepsBuildFiles = src: filterSrcWithRegexes [ "Cargo.lock" "Cargo.toml" ".*/Cargo.toml" ] src;

        # Filter only files relevant to building the workspace
        filterWorkspaceFiles = src: filterSrcWithRegexes [ "Cargo.lock" "Cargo.toml" ".*/Cargo.toml" ".*\.rs" ] src;

        filterSrcWithRegexes = regexes: src:
          let
            basePath = toString src + "/";
          in
          lib.cleanSourceWith {
            filter = (path: type:
              let
                relPath = lib.removePrefix basePath (toString path);
                includePath =
                  (type == "directory") ||
                  lib.any
                    (re: builtins.match re relPath != null)
                    regexes;
              in
              # uncomment to debug:
                # builtins.trace "${relPath}: ${lib.boolToString includePath}"
              includePath
            );
            inherit src;
          };

        commonArgs = {
          src = filterWorkspaceFiles ./services;
          buildInputs = [
          ];
          nativeBuildInputs = [
            pkgs.pkg-config
            fenix-channel.rustc
          ];
        };

        workspaceDeps = craneLib.buildDepsOnly (commonArgs // {
          src = filterWorkspaceDepsBuildFiles ./services;
          pname = "services-workspace-deps";
          doCheck = false;
        });

        workspaceBuild = craneLib.cargoBuild (commonArgs // {
          cargoArtifacts = workspaceDeps;
          doCheck = false;
        });

        workspaceTest = craneLib.cargoBuild (commonArgs // {
          cargoArtifacts = workspaceBuild;
          doCheck = true;
        });

        workspaceClippy = craneLib.cargoClippy (commonArgs // {
          cargoArtifacts = workspaceBuild;
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

        # external project would import `rustshop` as a flake,
        # but we cheat, at least for now; but it is an external
        # components, and working on it requires `cd rustshop; nix develop`
        # as it maintains it's own flake
        rustshop = (import ./rustshop/default-system.nix) system;
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

          inherit workspaceDeps workspaceBuild workspaceTest workspaceClippy;
        };

        legacyPackages = rustshop.legacyPackages.${system};

        devShells = {
          default =
            pkgs.mkShell {
              inputsFrom = [
                (rustshop.devShells.${system}.default.overrideAttrs
                  (prev: {
                    shellHook = ''
                      PATH="$PATH:${./.config/flakebox/bin}/"
                    '' + prev.shellHook;
                  }))
              ];
              buildInputs = workspaceDeps.buildInputs;
              nativeBuildInputs = workspaceDeps.nativeBuildInputs ++
                lib.attrsets.attrValues rustshop.packages.${system} ++ [

                # extra binaries here
                fenix-toolchain

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
                pkgs.cargo-udeps
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
          lint = rustshop.devShells.${system}.lint;
        };
      });
}
