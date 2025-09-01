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
    if (echo "$recipe" | grep '.*/.*' >/dev/null); then
        recipe_name=$(basename "$recipe")
        recipe_path="$recipe"
    else
        recipe_name="$recipe"
        recipe_path=`target/release/find_recipe $recipe`
    fi

    rm -rfv "$recipe_path"/source "$recipe_path"/source.tar
done

