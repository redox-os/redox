#!/usr/bin/env bash

set -e

git fetch origin master
packages=""
for toml in $(git diff --name-only origin/master... | grep '/recipe.toml$' | sort | uniq)
do
    package="$(basename "$(dirname "${toml}")")"
    if [ -n "${packages}" ]
    then
        packages="${packages},"
    fi
    packages="${packages}${package}"
done
if [ -n "${packages}" ]
then
    make f."${packages}"
else
    echo "No recipe.toml changes found"
fi


