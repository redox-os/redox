#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(ls -1 recipes)"
else
    recipes="$@"
fi

for recipe in $recipes
do
    if [ -e "recipes/$recipe/recipe.toml" ]
    then
        target/release/cook --fetch-only "$recipe"
        continue
    fi

    ./cook.sh "$recipe" fetch
done
