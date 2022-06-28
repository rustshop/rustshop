{ config, lib, pkgs, ... }:
with lib;
let
  # These options will be used for user account defaults in
  # the `mkUser` function.
  rustshop.users = {
    groups = mkOption {
      type = types.listOf types.str;
      default = [ "wheel" ];
      example = ''[ "wheel" "libvirtd" "docker" ]'';
      description =
        "The Unix groups that Xeserv staff users should be assigned to";
    };
    
    shell = mkOption {
      type = types.package;
      default = pkgs.bashInteractive;
      example = "pkgs.powershell";
      description =
        "The default shell that Xeserv staff users will be given by default.";
    };
  };
  
  cfg = config.rustshop.users;

  mkUser = {
    keys,
    shell ? cfg.shell,
    extraGroups ? cfg.groups,
    description ? "",
    ...
  }: {
    isNormalUser = true;
    inherit extraGroups shell description;
    openssh.authorizedKeys = {
      inherit keys;
    };
  };
in {
  options.rustshop.users = rustshop.users;
  
  config.users.users = {
    dpc = mkUser {
      shell = pkgs.fish;
      description = "Dawid Ciężarkiewicz";
      keys = [
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDZfZz0f8MwAE+Mto/jUwc3hWnjwDB3JNXB/Pnz5Ej6jJfY+YbBWqWTuAJ/sjRiYMpDQdDgmhBBkguFrGGVEujo5L7Pskwt7U6eRsnQGAJOjX2cd+vBworJ0m0dCbeTTPovk5ayHGQqW/9xSYqZCIsnkJpcKIeCll4M3d7mUsf+ZusxiKbrUFmyvqMPEHmuGpNRuMJlY1+b0kEYa/hE8HyP5L0arNdIylMfEyAPlBK9V8rD53NZpk19GFVN8m5J02Bhig74xAZAwGjo35Gol55kmFJ8rVWd+/Hyw5a8R23J2G5PsBhopGYJhpco1smvpAin4F+amIDecVvd0WVOEhxtnED3ZqyUhoYftVqCsIBRPBe3RIIHDtS3/IQv5gyS1tGfO7S7MSfbFoCLm5qLq4a1oY7D7NirY8e2hb4DrNIvuB7WkttF8zKCoEO+z9KzRGwBsNOPpOsayxOwCxWmd6UOu2iv8r7ibsUKZJiQbKvcJWe1Iix8rr51nY+CSVHzkHW7tsOUppptU6VKIVkhBkxdAPU88a1mhyGRAhdC0B3y7HGIhp83bCez8B0AHRe3YY0kiDRd/oR+6+bazne38Y/pZd7DNFUp7hpbnpPt/Cmx688FgnlIZETby74CE5S9W0i9uhwGaxR6c1XcOkt/wTvyZibVs5Dk0J7KEGYGANhLCQ== openpgp:0x83853072"
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCXIyNYNuIIy/jp1pc2ezMXfvxJ0Ng5awkEkwNzNfGMn5hn7qx0cT+Z9JtM4k22CTbWR+PYM4R8pc91lScKHjGtHA8QgVjgtAngOqwfIpF/R1wVB9QPQFdB+L5zBRbBzMm+sAe+yX5GnoxonxRqKioWZifGQqmpYVWeuJg64J2Xd2N9HqrtOGORCZUmN+5VfgC68UaOB/t+C/qORpngen6zKghgK9KJoO+mLfGWxjrjlSIfF9uoP6hFIL/j1r0tn6YaUV9b+UW2bN5qMGPhQja0ByZUysp+vI51pPJdXtVPYoE9AYR6vxRvsd2gVe5MrgkC4L6KubrhdW6BecH4RlWd cardno:000605313582"
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCfaPE4G+ndC8QVUEDhea/YItohes+9kEzdZmGk+Uv9KCzdu/ZbMAzv4Wu/MeLh6NeTTPjwWnh5PKzw7EWV4fk0+nnml230aFiveMZ9FAqKabV3oa6Xu3E1fHCoR38pllJ30Xbcmru9+0QMokZaJqoBxwaeuZLnG2D7PP1sdlNYEl8/YbnUBW7+8hnFeK+0+egobG6CaZApsBvDNr/cjliwQtfB0LofgpODZkMVj0H4EmxsaKDYrtxfGXvMAqIcbmWfM9xIZCjsUk759LK07lrg0veRXi8hHin807CUVfKGcrub7GDjsAkK1qbY4nA4ZSnUVBpVIqwtqJNhPIhDU+R/ openpgp:0x8AF85457"
      ];
    };
  };
}
