#!/usr/bin/env bash
set -e

source config.sh

if [ $# = 0 ]
then
    recipes="--all"
else
    recipes="$@"
fi

repo clean $recipes
