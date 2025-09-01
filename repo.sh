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

if [ "$recipes" == "" ]
then
    recipes="$(target/release/list_recipes)"
fi

for recipe in $recipes
do
    recipe_path=`target/release/find_recipe $recipe`
    COOKBOOK_RECIPE="$recipe_path"
    TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
    COOKBOOK_BUILD="${TARGET_DIR}/build"
    COOKBOOK_STAGE="${TARGET_DIR}/stage"
    COOKBOOK_SOURCE="${COOKBOOK_RECIPE}/source"
    COOKBOOK_SYSROOT="${TARGET_DIR}/sysroot"

    target/release/cook $COOK_OPT "$recipe"
done

mkdir -p "$REPO"

declare -A APPSTREAM_SOURCES

# Runtime dependencies include both `[package.dependencies]` and dynamically
# linked packages discovered by auto_deps.
#
# The following adds the package dependencies of the recipes to the repo as
# well.
recipes="$recipes $(target/release/pkg_deps $recipes)"

target/release/repo_builder "$REPO" $recipes
