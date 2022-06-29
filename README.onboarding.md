# Set up local dev environment

## Clone reposity

Clone this repository locally, with `git clone <repo-url>`, then `cd <repo-dir>`

## Set up Nix

### Install nix

Nix is used to manage local dev env requirements. If you don't have it set up already,
follow the instructions at: https://nixos.org/download.html

The end result is having a working `nix` command in your shell.

Example:

```
> nix --version
nix (Nix) 2.9.1
```

### Enable nix flakes

https://nixos.wiki/wiki/Flakes#Installing_flakes

The end result is having `nix develop` work (it might take
a while for it download all the neccessary files.
