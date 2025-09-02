#!/usr/bin/env bash
set -e

source config.sh

recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--nonstop" ]
    then
        set +e
    elif [ "$arg" == "--offline" ]
    then
        export COOKBOOK_OFFLINE="1"
    else
        recipes+=" $arg"
    fi
done

for recipe_name in $recipes
do
    target/release/cook --fetch-only "$recipe_name"
done
