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
    elif [ "$arg" == "--debug" ]
    then
        DEBUG=--debug
    elif [ "$arg" == "--with-package-deps" ]
    then
        COOK_OPT=--with-package-deps
    elif [ "$arg" == "--nonstop" ]
    then
        set +e
    elif [ "$arg" == "--offline" ]
    then
        export COOKBOOK_OFFLINE="1"
    else
        recipes+=" $arg"
    fi
done

for recipe in $recipes
do
    target/release/cook $COOK_OPT "$recipe"
done

repo="$ROOT/repo/$TARGET"
mkdir -p "$repo"

# Runtime dependencies include both `[package.dependencies]` and dynamically
# linked packages discovered by auto_deps.
#
# The following adds the package dependencies of the recipes to the repo as
# well.
recipes="$recipes $(target/release/pkg_deps $recipes)"

target/release/repo_builder "$repo" $recipes
