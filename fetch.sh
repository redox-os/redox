#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(target/release/list_recipes)"
else
    recipes="$@"
fi

for recipe in $recipes
do
    recipe_path=`target/release/find_recipe $recipe`
    if [ -e "$recipe_path/recipe.toml" ]
    then
        target/release/cook --fetch-only "$recipe"
        continue
    fi

    ./cook.sh "$recipe" fetch
done
