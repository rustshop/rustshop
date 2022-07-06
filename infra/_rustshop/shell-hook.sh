#!/usr/bin/env sh
#
# Shell hook to initialize rustshop's environment

export RUSTSHOP_ROOT="`pwd`"

# import (and export) all envs from `.env`
if [ -e ".env" ]; then
  set -a
  . ./.env
  set +a
fi

if [ -z "$RUSTSHOP_NAME" ]; then
  echo '"RUSTSHOP_NAME" not set. Edit .env and try again.' 1>&2
  exit 1
fi

if [ -z "$RUSTSHOP_DOMAIN" ]; then
  echo '"RUSTSHOP_DOMAIN" not set. Edit .env and try again.' 1>&2
  exit 1
fi

# execute local customization script
. "$RUSTSHOP_ROOT/.shrc"

alias k=kubectl

# Completions
eval "`aws-bootstrap --completions \`basename $SHELL\``"
# Completions
eval "`rustshop --completions \`basename $SHELL\``"
