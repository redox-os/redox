# This script create a list with:
# "recipe-name = {} #TODO"
# For quick testing of WIP recipes

#!/usr/bin/env bash

# Given a string, find recipe.toml files containing that string.
# Create a list that can be copy/pasted into a filesystem config.

if [ -z "$*" ]
then
    echo "Find matching recipes, and format for inclusion in config"
    echo "Usage: $0 \"pattern\""
    echo "Must be run from 'redox' directory"
    echo "e.g. $0 \"TODO.*error\""
    exit 1
fi

cookbook_recipes="cookbook/recipes"
recipe_paths=$(grep -rl "$*" "$cookbook_recipes" --include recipe.toml)

for recipe_path in $recipe_paths
do
    recipe_dir="$(dirname $recipe_path)"
    recipe_name="$(basename $recipe_dir)"
    echo "$recipe_name = {}    # " $(grep "$*" $recipe_path)
done