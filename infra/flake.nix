{
  description = "Infra";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat}:
    flake-utils.lib.eachDefaultSystem (system:
    let
      imp = (import ./utils/aws-bootstrap/default.nix);
      overlay = self: super: {
        aws-bootstrap = builtins.trace ''${imp}'' imp.default;
      };
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ overlay ];
      };
    in {
      devShell = pkgs.mkShell {
        buildInputs = [
          pkgs.terraform
          pkgs.awscli2
          pkgs.aws-bootstrap
        ];
      };
  });
}
