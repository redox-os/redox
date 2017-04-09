#!/bin/bash

set -e

if [ $# = 0 ]
then
    recipes=$(ls -1 recipes)
else
    recipes=$@
fi

publish=""
for recipe in $recipes
do
    if [ ! -f "recipes/$recipe/stage.tar" ]
    then
        echo "$recipe: building..."
        ./cook.sh $recipe dist
        publish="${publish} $recipe"
    else
        oldver=$(COOK_QUIET=1 ./cook.sh $recipe gitversion)
        ./cook.sh $recipe fetch
        newver=$(COOK_QUIET=1 ./cook.sh $recipe gitversion)
        if [ "$oldver" = "$newver" ]
        then
            echo "$recipe: up to date (version $newver)."
        else
            echo "$recipe: updating $oldver -> $newver..."
            ./cook.sh $recipe unstage untar dist
            publish="${publish} $recipe"
        fi
    fi
done

for recipe in $publish
do
    ./cook.sh $recipe publish
done
