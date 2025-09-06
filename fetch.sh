#!/usr/bin/env bash
set -e

source config.sh

recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--nonstop" ]
    then
        set +e
    elif [ "$arg" == "--offline" ]
    then
        export COOKBOOK_OFFLINE="1"
    else
        recipes+=" $arg"
    fi
done

if [ "$recipes" == "" ]
then
    recipes="$(target/release/list_recipes)"
fi

for recipe_path in $recipes
do
    if (echo "$recipe_path" | grep '.*/.*' >/dev/null); then
        recipe_name=$(basename "$recipe_path")
        recipe_path="$recipe_path"
    else
        recipe_name="$recipe_path"
        recipe_path=`target/release/find_recipe $recipe_name`
    fi

    target/release/cook --fetch-only "$recipe_name"
done
