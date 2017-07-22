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
    elif [ -e "recipes/$recipe/source.tar" ]
    then
        echo "Using source tarball"
        tar --compare --file="recipes/$recipe/source.tar" -C "recipes/$recipe/source" --strip-components=1 2>&1| grep -v "tar: :" | grep -v '\(Mode\|Gid\|Uid\) differs' || true
    else
        echo "No original source found"
    fi
done
