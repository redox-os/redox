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
    echo -e "\e[1m$recipe\e[0m"
    if [ -d "recipes/$recipe/source/.git" ]
    then
        git -C "recipes/$recipe/source" status
    else
        echo "Not a GIT repository"
    fi
done
