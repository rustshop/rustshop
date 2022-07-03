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
      fenix-channel = fenix-pkgs.stable;

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

      deps = craneLib.buildDepsOnly (commonArgs // {
        pname = "deps";
      });

      aws-bootstrap = craneLib.buildPackage (commonArgs // {
        pname = "aws-bootstrap";
      });

      rustshop-bin-wrapper = craneLib.buildPackage (commonArgs // {
        pname = "rustshop-bin-wrapper";
      });

    in {
      packages = {
        aws-bootstrap = aws-bootstrap;
        rustshop-bin-wrapper = rustshop-bin-wrapper;
      };

      devShell = pkgs.mkShell {
        buildInputs = deps.buildInputs;
        nativeBuildInputs = deps.nativeBuildInputs ++ [
          fenix-pkgs.rust-analyzer
          fenix-channel.rustfmt
          fenix-channel.rustc
          fenix-channel.cargo
        ];
        RUST_SRC_PATH = "${fenix-channel.rust-src}/lib/rustlib/src/rust/library";
      };
  });
}
