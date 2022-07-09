# `rustshop` (the binary)

This is the core tool of `rustshop` (the project).

It can;

* bootstrap aws accounts and k8s clusters
* wrap tools like `aws` CLI, `terraform`, `kops`, `kubectl` and other to enhance them
This binary is used to wrap all the typical utilities used with aws cli



# Wrapping

When used for wrapping it will automatically inject inject relevant
environment variables, depending on a current current rustshop context
(shop&user information about AWS accounts and clusters).

It also takes care of any small fixups and workaround to achieve smooth
experience when working with rustshop.

Nix shell is used to conveniently wrap a normal binary and replace it with
a call:

```
exec -a "$0" rustshop wrap actual-binary "$@"
```

A `RUSTSHOP_NO_BIN_WRAP=true` env flag can be used to make `rustshop`
not alter the execution of the wrapped binary.
