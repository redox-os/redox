#!/usr/bin/env bash

set -ex

if [ -z "$1" ]
then
    echo "$0: no argument provided"
    exit 1
fi

if [ ! -d "cookbook/recipes/$1" ]
then
    echo "$0: $1 is not a recipe"
    exit 1
fi

rm -rfv "cookbook/recipes/$1/"{source,source.tar,target}
make "r.$1"
