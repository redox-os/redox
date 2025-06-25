#!/usr/bin/env bash

# This script run the recipe command options on some Cookbook category

if [ -z "$1" ] || [ -z "$2" ]
then
    echo "Build or clean all recipe directories in a category" >&2
    echo Usage: $0 "<action>" "<recipe-category>" >&2
    echo "<action>" can be f, r, c, u, or combinations that \"make\" understands >&2
    exit 1
fi

action="${1#-}"

recipe_list=""
first=1

for recipe in `find cookbook/recipes/"$2" -name "recipe.*"`
do
    recipe_folder=`dirname "$recipe"`
    recipe_name=`basename "$recipe_folder"`
    if [ "$first" -eq 1 ]; then
        recipe_list="$recipe_name"
        first=0
    else
        recipe_list="$recipe_list,$recipe_name"
    fi
done

set -x
make "$action"l."$recipe_list"
