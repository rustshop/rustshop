# Services

This directory is a Nix workspace for all application code
in our shop.

Most directories are either application support/utility libraries or
applications itself.

## Using

To get a dev env (Nix shell), run:

```sh
nix develop
```

To build you can either use normal `cargo build ...` inside
the Nix shell, or use:

```sh
nix build .#starter
```

You can run with normal `cargo run ...` or run:

```sh
nix run .#starter
```

To build a docker container use:

```
nix build .#starter
docker load < result
```

(replace `starter` with the application in question)

Check [`flake.nix`](./flake.nix) for list of services.
