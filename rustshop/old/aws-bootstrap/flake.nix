{
  description = "AWS Bootstrap - simple utility to bootstrap AWS account for self-hosted Terraform";

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

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "aws-bootstrap-deps";
        });

        aws-bootstrap = craneLib.buildPackage (commonArgs // {
          pname = "aws-bootstrap";
        });

      in
      {
        aws-bootstrap = aws-bootstrap;
        defaultPackage = aws-bootstrap;

        devShell = pkgs.mkShell {
          buildInputs = cargoArtifacts.buildInputs;
          nativeBuildInputs = cargoArtifacts.nativeBuildInputs ++ [
            fenix-pkgs.rust-analyzer
            fenix-channel.rustfmt
            fenix-channel.rustc
            fenix-channel.cargo
          ];
          RUST_SRC_PATH = "${fenix-channel.rust-src}/lib/rustlib/src/rust/library";
        };
      });
}
