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
    if [ -d "recipes/$recipe/source" ]
    then
        status="$(COOK_QUIET=1 ./cook.sh "$recipe" status_upstream)"

        if [ -n "$status" ]
        then
            echo -e "\e[1m$recipe\e[0m\n$status"
        fi
    fi
done
