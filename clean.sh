#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(target/release/list_recipes)"
else
    recipes="$@"
fi

for recipe_path in $recipes
do
    IGNORE_ERROR=1 ./cook.sh "$recipe_path" distclean
done
