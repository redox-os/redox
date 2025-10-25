#!/usr/bin/env bash
set -e

source `dirname "$0"`/config.sh

if [ $# = 0 ]
then
    recipes="--all"
else
    recipes="$@"
fi

repo clean $recipes
