{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, nixos-generators, ... }:
  let
    trimSizeModule = {lib, system, boot, ...}: {
      imports = [
        (import "${nixpkgs}/nixos/modules/profiles/minimal.nix")
      ];
      swapDevices = lib.mkForce [ ];

      programs.command-not-found.enable = lib.mkDefault false;
      services.udisks2.enable = false;
      boot.enableContainers = false;
      environment.defaultPackages = [];
      # https://github.com/nix-community/nixos-generators/issues/164
      # system.disableInstallerTools = true;
    };
  in {
    packages.x86_64-linux = {
      amazon = nixos-generators.nixosGenerate {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;
        modules = [
          trimSizeModule
          ./configuration.nix
        ];
        format = "amazon";
      };
      vbox = nixos-generators.nixosGenerate {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;
        modules = [
          trimSizeModule
          ./configuration.nix
        ];
        format = "virtualbox";
      };
    };
  };
}
