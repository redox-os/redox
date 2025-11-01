#!/usr/bin/env bash
set -e

source `dirname "$0"`/config.sh

APPSTREAM="0"
COOK_OPT=""
recipes=""
for arg in "${@:1}"
do
    if [[ "$arg" == "--appstream" ]]
    then
        APPSTREAM="1"
    elif [[ "$arg" == "--offline" ]]
    then
        export COOKBOOK_OFFLINE=true
    elif [[ $arg == "--*" ]]
    then
        COOK_OPT+=" ${arg}"
    else
        recipes+=" $arg"
    fi
done

repo cook $COOK_OPT $recipes

repo_builder "$ROOT/repo/$TARGET" $recipes
