# Binary wrapper for `rustshop`


This binary is used to wrap all the typical utilities used with aws cli
(`aws`, `terraform`, `kops`, etc.`), and automatically inject relevant
environment variables, depending on a current directory (`pwd`).

It also takes care of any small fixups and workaround to achieve smooth
experience when working with rustshop.

Nix shell is used to conveniently wrap a normal binary and replace it with
a call:

```
exec -a "$0" rustshop-bin-wrapper actual-binary "$@"
```

A `RUSTSHOP_BIN_NO_WRAP=true` env flag can be used to avoid any 
