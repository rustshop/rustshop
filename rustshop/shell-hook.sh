#!/usr/bin/env sh
#
# Standard shell hook to initialize shop's shell to use `rustshop`

export RUSTSHOP_ROOT

RUSTSHOP_ROOT=$(pwd)

# execute shop-wide customization script
if [ -e "$RUSTSHOP_ROOT/.rustshop/shop.shrc" ]; then
. "$RUSTSHOP_ROOT/.rustshop/shop.shrc"
fi

# execute user-local customization script
if [ -e "$RUSTSHOP_ROOT/.rustshop/user.shrc" ]; then
. "$RUSTSHOP_ROOT/.rustshop/user.shrc"
fi

# Completions
eval "$(rustshop --completions "$(basename "$SHELL")")"
