#!/usr/bin/env bash

# This script runs "make f.recipe" and "cargo update" in the specified recipe

recipe_name="$1"
recipe_path=$(find cookbook/recipes -name "$recipe_name" -maxdepth 4)

make f."$recipe_name"
cd "$recipe_path"/source
cargo update
