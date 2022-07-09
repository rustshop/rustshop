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

      workspaceDeps = craneLib.buildDepsOnly (commonArgs // {
        pname = "services-workspace-deps";
      });

      # a function to define both package and docker build for a given service binary
      servicesApp = name: rec {
        package = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = workspaceDeps;
          pname = name;

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

      apps = {
        starter = servicesApp "starter";
      };

    in {
      packages = rec {
        default = starter;

        starter = apps.starter.package;

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
