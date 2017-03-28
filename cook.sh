#!/bin/bash

# Configuration
export TARGET=x86_64-unknown-redox

# Automatic variables
ROOT="$(cd `dirname "$0"` && pwd)"
REPO="$ROOT/repo/$TARGET"
export CC="$ROOT/libc-artifacts/gcc.sh"

# Variables to be overriden by recipes
export BINDIR=bin
export CARGOFLAGS=

set -e

function op {
    echo -e "\033[01;38;5;215mcook - $1 $2\033[0m"
    case "$2" in
        dist)
            op $1 fetch
            op $1 update
            op $1 build
            op $1 stage
            op $1 tar
            ;;
        distclean)
            op $1 untar
            op $1 unstage
            op $1 unfetch
            ;;
        fetch)
            if [ ! -d build ]
            then
                git clone --recursive "$GIT" build
            fi

            pushd build > /dev/null
            git pull
            git submodule sync
            git submodule update --init --recursive
            popd > /dev/null
            ;;
        unfetch)
            rm -rfv build
            ;;
        info)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_info)" = "function" ]
            then
                recipe_info || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                echo "$1_$(cargo config package.version | tr -d '"')"
            fi
            popd > /dev/null
            ;;
        update)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_update)" = "function" ]
            then
                recipe_update || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                xargo update
            fi
            popd > /dev/null
            ;;
        build)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_build)" = "function" ]
            then
                recipe_build || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                cp -r "$ROOT/Xargo.toml" "$ROOT/.cargo" "$ROOT/libc-artifacts" .
                xargo build --target "$TARGET" --release $CARGOFLAGS
            fi
            popd > /dev/null
            ;;
        test)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_test)" = "function" ]
            then
                recipe_test || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                cp -r "$ROOT/Xargo.toml" "$ROOT/.cargo" "$ROOT/libc-artifacts" .
                xargo test --no-run --target "$TARGET" --release $CARGOFLAGS
            fi
            popd > /dev/null
            ;;
        clean)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_clean)" = "function" ]
            then
                recipe_clean || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                xargo clean
            fi
            popd > /dev/null
            ;;
        stage)
            mkdir -p stage
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_stage)" = "function" ]
            then
                recipe_stage ../stage || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                #TODO xargo install --root "../stage" $CARGOFLAGS
                bins="$(find target/$TARGET/release/ -maxdepth 1 -type f ! -name '*.*')"
                if [ -n "$bins" ]
                then
                    mkdir -p "../stage/$BINDIR"
                    for bin in $bins
                    do
                        cp -v "$bin" "../stage/$BINDIR/$(basename $bin)"
                        strip -v "../stage/$BINDIR/$(basename $bin)"
                    done
                fi
            fi
            popd > /dev/null
            ;;
        unstage)
            rm -rfv stage
            ;;
        tar)
            pushd stage > /dev/null
            tar cfv ../stage.tar .
            popd > /dev/null
            ;;
        untar)
            rm -rfv stage.tar
            ;;
        publish)
            mkdir -p "$REPO"
            cp -v stage.tar "$REPO/$1.tar"
            ;;
        unpublish)
            rm -rfv "$REPO/$1.tar"
            ;;
        *)
            echo "cook.sh $1 {dist|distclean|build|clean|fetch|unfetch|publish|unpublish|stage|unstage|tar|untar|update}"
            ;;
    esac
}

if [ -n "$1" ]
then
    if [ -d "$ROOT/recipes/$1" ]
    then
        cd "$ROOT/recipes/$1"
        source recipe.sh
        for arg in "${@:2}"
        do
            op "$1" "$arg"
        done
    else
        echo "cook.sh: recipe '$1' not found"
    fi
else
    echo "cook.sh {package} {dist|distclean|build|clean|fetch|unfetch|publish|unpublish|stage|unstage|tar|untar|update}"
fi
