#!/usr/bin/env sh
#
# Flake Shell Hook - executed on start on every `nix develop`,
# to set up some stuff. Called from `mkShell` `shellHook` in `flake.nix`

export PATH="$PWD/utils:$PATH"

alias terraform='terraform-wrapper'

# Initialize variable files from templates
if [ ! -e ".env" ]; then
  echo 'Creating .env' 1>&2
  cp .env.template .env
fi
if [ ! -e ".shrc" ]; then
  echo 'Creating .shrc' 1>&2
  cp .shrc.template .shrc
fi

# import (and export) all envs from `.env`
set -a
. ./.env
set +a

if [ -z "$TF_VAR_SHOPNAME" ]; then
  echo '"TF_VAR_SHOPNAME" not set. Edit .env and try again.' 1>&2
  exit 1
fi

if [ -z "$TF_VAR_AWS_ACCOUNT_ID_ROOT" ]; then
  echo '"TF_VAR_AWS_ACCOUNT_ID_ROOT" not initialized. Edit variables in .env after creating AWS accounts with `aws-bootstrap`.' 1>&2
fi

if [ -z "$AWS_PROFILE" ]; then
  export AWS_PROFILE="${TF_VAR_SHOPNAME}-root"
  echo "Setting AWS_PROFILE=${AWS_PROFILE}" 1>&2
fi

# execute local customization script
. ./.shrc
