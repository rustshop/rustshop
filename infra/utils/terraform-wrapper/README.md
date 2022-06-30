# Custom `terraform` wrapper for `rustshop`

This project exists entirely to work around a limitation of `terraform init`
being unable to use input variables.

In `rustshop` we don't want to hardcode any account data, etc. since it is
a public repository, and we care about usability.

Because of this we use `alias terraform=terraform-wrapper` in the Nix Shell
(`nix develop`), and detect `terraform init` invocation, then apply the workaround
from https://github.com/hashicorp/terraform/issues/13022#issuecomment-294262392 ,
with env values from the Nix Shell (`.env` file)..
