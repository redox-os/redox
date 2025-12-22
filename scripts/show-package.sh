#!/usr/bin/env bash

# This script show the contents of the "stage" and "sysroot" folders in some recipe

if [ -z "$*" ]
then
    echo "Show the contents of the stage and sysroot folders in recipe(s)"
    echo "Usage: $0 recipe1 ..."
    echo "Must be run from the 'redox' directory"
    echo "e.g. $0 kernel"
    exit 1
fi

find_recipe="target/release/find_recipe"
if [ ! -x "$find_recipe" ]
then
    echo "$find_recipe not found."
    echo "Please run 'make fstools' and try again."
    exit 1
fi

for recipe in $*
do
    recipe_dir="$("$find_recipe" "$recipe")"
    ls -1 "$recipe_dir/target"/*/{stage,sysroot}
done
