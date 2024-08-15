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
    if (echo "$recipe_path" | grep '.*/.*' >/dev/null); then
        recipe_name=$(basename "$recipe_path")
        recipe_path="recipes/$recipe_path"
    else
        recipe_name="$recipe_path"
        recipe_path=`target/release/find_recipe $recipe_name`
    fi

    if [ -e "$recipe_path/recipe.toml" ]
    then
        target/release/cook --fetch-only "$recipe_name"
    else
        ./cook.sh "$recipe_name" fetch
    fi
done
