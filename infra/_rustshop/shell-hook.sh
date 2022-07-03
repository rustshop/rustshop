#!/usr/bin/env sh
#
# Shell hook to initialize rustshop's environment

# import (and export) all envs from `.env`
if [ -e ".env" ]; then
  set -a
  . ./.env
  set +a
fi

if [ -z "$RUSTSHOP_NAME" ]; then
  echo '"RUSTSHOP_SHOPNAME" not set. Edit .env and try again.' 1>&2
  exit 1
fi

# execute local customization script
. "$RUSTSHOP_ROOT/.shrc"

# Completions
eval "`aws-bootstrap --completions \`basename $SHELL\``"
