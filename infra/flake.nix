{
  description = "Infra";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    # aws-bootstrap = {
    #   url = "./utils/aws-bootstrap/";
    # };
    # aws-bootstrap.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat}:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      aws-bootstrap = pkgs.callPackage ./utils/aws-bootstrap/default.nix {};
    in {

      devShell = pkgs.mkShell {
        buildInputs = [
          pkgs.terraform
          pkgs.awscli2
          aws-bootstrap
        ];
      };
  });
}
