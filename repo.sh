#!/usr/bin/env bash
set -e

source `dirname "$0"`/config.sh

APPSTREAM="0"
COOK_OPT=""
recipes=""
for arg in "${@:1}"
do
    if [ "$arg" == "--appstream" ]
    then
        APPSTREAM="1"
    elif [ "$arg" == "--with-package-deps" ]
    then
        COOK_OPT+=" --with-package-deps"
    elif [ "$arg" == "--nonstop" ]
    then
        export COOKBOOK_OFFLINE=true
    else
        recipes+=" $arg"
    fi
done

repo cook $COOK_OPT $recipes

repo_builder "$repo" $recipes
