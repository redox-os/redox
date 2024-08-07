#!/usr/bin/env bash

# This script print all recipe executable names to find duplicates and verify executable name conflicts

usage() {
    echo "List executable names to find duplicates"
    echo "Usage: $0 [-h] [-a] [-A | -6] [recipes]"
    echo "Default architecture is x86_64, -A is aarch64, -6 is i686"
    echo "Only duplicates are listed unless -a is specified"
    echo "-h is this message"
    exit
}

cd cookbook

recipes=""

target="x86_64-unknown-redox"
uniq="uniq -D --skip-fields=1"

for arg in "${@:1}"
do
    if [ "$arg" == "-A" ]
    then
        target="aarch64-unknown-redox"
    elif [ "$arg" == "-6" ]
    then
        target="i686-unknown-redox"
    elif [ "$arg" == "-a" ]
    then
        uniq="cat"
    elif [ "$arg" == "-h" ]
    then
    	usage
    else
        recipes+=" $arg"
    fi
done

if [ -z "$recipes" ]
then
    recipes="$(target/release/list_recipes)"
fi

for recipe in $recipes
do
    if [[ "$recipe" == *\/* ]]
    then
        recipe_name="$(basename $recipe)"
        recipe_path="recipes/$recipe"
    else
        recipe_name="$recipe"
        recipe_path="$(target/release/find_recipe $recipe_name)"
    fi
    
    for command in $(find "$recipe_path/target/$target/stage/usr/bin" -type f 2> /dev/null) \
        $(find "$recipe_path/target/$target/stage/bin" -type f 2> /dev/null)
    do
        shortname="$(basename $command)"
    	echo "$recipe_path $shortname"
    done
done | sort | $uniq
