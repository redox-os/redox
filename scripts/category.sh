# This script run the recipe command options on some Cookbook category

#!/usr/bin/env bash

if [ -z "$1" ] || [ -z "$2" ]
then
    echo "Build or clean all recipe directories in a category" >&2
    echo Usage: $0 "<action>" "<recipe-category>" >&2
    echo "<action>" can be f, r, c, u, or combinations that \"make\" understands >&2
    exit 1
fi

set -x

action="${1#-}"

for recipe in `find cookbook/recipes/"$2" -name "recipe.*"`
do
    recipe_folder=`dirname "$recipe"`
    recipe_name=`basename "$recipe_folder"`
    make "$action"."$recipe_name"
done