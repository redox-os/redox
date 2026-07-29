#!/usr/bin/env bash

# This script print the recipe configuration

cat $(make find.$1)/recipe.toml
