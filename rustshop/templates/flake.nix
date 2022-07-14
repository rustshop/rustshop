{
  description = "Shop monorepo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rustshop = {
      url = "github:rustshop/rustshop?dir=rustshop";
    };

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

  outputs = { self, naersk, nixpkgs, flake-utils, flake-compat, fenix, crane, rustshop }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        lib = nixpkgs.lib;
      in
      {
        packages = { };

        devShells.default = pkgs.mkShell {
          buildInputs =
            lib.attrsets.attrValues rustshop.packages."${system}" ++ [
              # extra binaries here
            ];

          shellHook = ''
            . ${rustshop.packages."${system}".rustshop}/usr/share/rustshop/shell-hook.sh
          '';
        };
      });
}
