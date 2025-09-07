#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

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
        COOK_OPT+=" --nonstop"
    elif [ "$arg" == "--offline" ]
    then
        COOK_OPT+=" --offline"
    else
        recipes+=" $arg"
    fi
done

target/release/cook $COOK_OPT $recipes

repo="$ROOT/repo/$TARGET"
mkdir -p "$repo"

target/release/repo_builder "$repo" $recipes
