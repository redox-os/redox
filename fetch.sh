#!/usr/bin/env bash -e

source config.sh

if [ $# = 0 ]
then
    recipes="$(ls -1 recipes)"
else
    recipes="$@"
fi

for recipe in $recipes
do
    ./cook.sh "$recipe" fetch
done
