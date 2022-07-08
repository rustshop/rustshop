# Set up local dev environment

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


And that's kind of it. From now one the Nix automation should
take care of everything for you.

## Clone repository

Clone this repository locally, with `git clone <repo-url>`, then `cd <repo-dir>`,

If your Nix is set up properly `nix develop` should just work (though it might take
a while to download all the necessary files and build all the internal tooling). In
the meantime you can read other documentation.


## Account setup (TODO)

Eventually we will have some "employee" management system that the CI/CD, infra access etc.
will rely on. Possibly the first PR each new employee does is some automatically generated
user-data file including their PGP key, github username, etc.

