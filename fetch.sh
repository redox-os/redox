#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(target/release/list_recipes)"
else
    recipes="$@"
fi

for recipe_path in $recipes
do
    if [ -e "$recipe_path/recipe.toml" ]
    then
        target/release/cook --fetch-only "$recipe_path"
    else
        ./cook.sh "$recipe_path" fetch
    fi
done
