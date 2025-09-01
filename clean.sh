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

    echo -e "\033[01;38;5;215mcook - clean $recipe_name\033[0m"

    if [ -d "$ROOT/$recipe_path" ]
    then
        COOKBOOK_RECIPE="${ROOT}/$recipe_path"
        TARGET_DIR="${ROOT}/$recipe_path/target/${TARGET}"

        rm -rf "${TARGET_DIR}"
    else
        echo "clean.sh: recipe '$recipe_name' not found" >&2
    fi
done
