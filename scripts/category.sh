#!/usr/bin/env bash

# This script run the recipe command options on some Cookbook category

if [ -z "$1" ] || [ -z "$2" ]
then
    echo "Build or clean all recipe directories in a category" >&2
    echo Usage: $0 "<action>" "<recipe-category>" >&2
    echo "<action>" can be f, r, c, u, p, or combinations that \"make\" understands >&2
    echo "<category>" can be path of category you want to run e.g. \"core\", \"wip\", \"wip/dev\" >&2
    exit 1
fi

make "${1#-}"."--category-$2"
