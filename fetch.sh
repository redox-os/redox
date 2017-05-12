#!/bin/bash -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(ls -1 recipes)"
else
    recipes="$@"
fi

for recipe in $recipes
do
    echo -e "\033[01;38;5;215mfetch - fetching $recipe\033[0m" >&2
    ./cook.sh "$recipe" fetch
done
