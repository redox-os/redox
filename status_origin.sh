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
    if [ -d "$recipe_path/source" ]
    then
        status="$(COOK_QUIET=1 ./cook.sh "$recipe" status_origin)"

        if [ -n "$status" ]
        then
            echo -e "\e[1m$recipe\e[0m\n$status"
        fi
    fi
done
