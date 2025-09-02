#!/usr/bin/env bash
set -e

source config.sh

recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--offline" ]
    then
        export COOKBOOK_OFFLINE="1"
    else
        recipes+=" $arg"
    fi
done

target/release/cook --fetch-only $recipes
