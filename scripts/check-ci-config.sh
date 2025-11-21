#!/usr/bin/env bash

set -e

if [ -n "$1" ]
then
    ARCH="$1"
else
    ARCH="x86_64"
fi

make build/fstools

declare -A packages
for recipe_dir in $(build/fstools/bin/list_recipes | grep -v '^recipes/wip/')
do
    recipe_name="$(basename "${recipe_dir}")"
    packages["${recipe_name}"]="${recipe_dir}"
done

config="config/${ARCH}/ci.toml"
for package in $(build/fstools/bin/redox_installer --list-packages -c "${config}")
do
    packages["${package}"]=""
done

echo "Checking for missing packages in ${config}"
printf '%-32s%s\n' "PACKAGE" "RECIPE"
for package in "${!packages[@]}"
do
    recipe_dir="${packages["${package}"]}"
    if [ -n "${recipe_dir}" ]
    then
        printf '%-32s%s\n' "${package}" "${recipe_dir}"
    fi
done | sort
