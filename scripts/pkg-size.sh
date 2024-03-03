#!/usr/bin/env bash

if [ $# = 0 ]
then
    find cookbook/recipes \( -name stage.pkgar -o -name stage.tar.gz \) -exec ls -hs {} \;
    exit 0
fi

for recipe in $@
do
    if [ "$recipe" = "-h" ] || [ "$recipe" = "--help" ]
    then
        echo "Usage: $0 [recipe] ..."
        echo "       For the recipe(s), prints the size of 'stage.pkgar' and 'stage.tar.gz'."
        echo "       If no recipe is given, then all packages are listed."
        exit 0
    fi

    recipe_paths=$(find cookbook/recipes -name $recipe)
    for recipe_path in $recipe_paths
    do
        if [ -f "$recipe_path/recipe.toml" ] || [ -f "$recipe_path/recipe.sh" ]
        then
            find "$recipe_path" \( -name stage.pkgar -o -name stage.tar.gz \) -exec ls -hs {} \;
        fi
    done
done