#!/usr/bin/env bash

# This script open the recipe configuration files with chosen EDITOR env, or chosen by "open" cli.

# This script must be launched outside podman

EDITOR=${EDITOR:-open}

$EDITOR $(make find.$1)/recipe.toml
