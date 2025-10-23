#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(list_recipes --short)"
else
    recipes="$@"
fi

for recipe_name in $recipes
do
    recipe_path=`find_recipe $recipe_name`

    echo -e "\033[01;38;5;215mcook - unfetch $recipe_name\033[0m"
    rm -rfv "$recipe_path"/source "$recipe_path"/source.tar
done

