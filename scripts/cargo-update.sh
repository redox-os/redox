#!/usr/bin/env bash

# This script runs "make f.recipe" and "cargo update" in the specified recipe

recipe_name="$1"
recipe_path=$(target/release/repo find $recipe_name)

make f."$recipe_name"
cd "$recipe_path"/source
cargo update
