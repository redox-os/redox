#!/usr/bin/env bash

set -e

# This script open the recipe configuration files with chosen EDITOR env, or chosen by "xdg-open" cli.

# This script must be launched outside podman

if [ -z "$1" ]
then
    echo "Open a file into a recipe."
    echo ""
    echo "Usage: [EDITOR=program] $0 <recipe-name> [file]"
    echo "  [EDITOR=program]: The EDITOR envar to set which app to open, default to 'EDITOR=xdg-open'"
    echo "  <recipe-name>: The recipe name, cannot be multiple"
    echo "  [file]: The filename to open, default to 'recipe.toml'"
    echo ""
    echo "example:"
    echo "  $0 kernel"
    echo "  EDITOR=nano $0 kernel"
    echo "  EDITOR=nvim $0 kernel source"
    exit 1
fi

EDITOR=${EDITOR:-xdg-open}
FILE=${2:-recipe.toml}

RECIPE_PATH=$(make find.$1)
RECIPE_PATH=${RECIPE_PATH//$'\r'/}
"$EDITOR" "$RECIPE_PATH/$FILE"
