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


## Use Nix Shell

If your Nix is set up properly `nix develop` should just work (though it might take
a while to download all the necessary files and build all the internal tooling). In
the meantime you can read other documentation.

Note: **using `nix develop` is virtually mandatory**. It takes care of setting up
all the required developer automation, checks and ensures that all the developers and CI are 
in sync: working with same set of tools (to the exact versions).

You can still use your favorite IDE, unix shell (you can make `nix develop` to `exec zsh`
etc. with `~/.rustshop/user.shrc`, and other personal utilities, but they MUST never
be expected for other developers to use. In other words: if it's not automated and
set up in `nix develop` shell, it doesn't exist.

## Account setup (TODO)

Eventually we will have some "employee" management system that the CI/CD, infra access etc.
will rely on. Possibly the first PR each new employee does is some automatically generated
user-data file including their PGP key, github username, etc.

