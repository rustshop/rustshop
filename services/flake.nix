{
  description = "Our services";

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
        src = ./.;
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
          lib.cleanSourceWith  {
          filter = (path: type:
            let
              relPath = lib.removePrefix basePath (toString path);
              includePath =
                (type == "directory" && builtins.match "^[^/]+$" relPath != null) ||
                lib.any
                  (re: builtins.match re relPath != null)
                  (["Cargo.lock" "Cargo.toml" ".*/Cargo.toml"] ++ builtins.concatLists (map (name: [name "${name}/.*"]) modules));
            in
              # uncomment to debug:
              # builtins.trace "${relPath}: ${lib.boolToString includePath}"
              includePath
          );
          src = ./.;
        };

      workspaceDeps = craneLib.buildDepsOnly (commonArgs // {
        pname = "services-workspace-deps";
      });

      # a function to define both package and docker build for a given service binary
      serviceApp = name: rec {
        package = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = workspaceDeps;
          pname = name;

          src = filterModules ["common-app" name] ./.;

          cargoExtraArgs = "--bin ${name}";
        });

        docker = pkgs.dockerTools.buildLayeredImage {
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

        src = filterModules ["common-res-gen" name] ./.;

        cargoExtraArgs = "--bin ${name}";
      });

      apps = {
        starter = serviceApp "starter";
      };

      resGens = {
        starter = resGen "starter";
      };

    in {
      packages = {
        default = apps.starter.package;

        app = {
          starter = apps.starter.package;
        };

        res = {
          starter = resGens.starter;
        };

        docker = {
          starter = apps.starter.docker;
        };
      };

      devShell = pkgs.mkShell {
        buildInputs = workspaceDeps.buildInputs;
        nativeBuildInputs = workspaceDeps.nativeBuildInputs ++ [
          fenix-pkgs.rust-analyzer
          fenix-channel.rustfmt
          fenix-channel.rustc
          fenix-channel.cargo
        ];
        RUST_SRC_PATH = "${fenix-channel.rust-src}/lib/rustlib/src/rust/library";
      };
  });
}
