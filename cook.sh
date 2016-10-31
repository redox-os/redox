#!/bin/bash

export RUST_TARGET_PATH=`realpath targets`
export RUSTFLAGS="--cfg redox"
export CARGOFLAGS=
TARGET=x86_64-unknown-redox

set -e

if [ -n "$1" ]
then
    if [ -d "recipes/$1" ]
    then
        pushd "recipes/$1"
        source recipe.sh
        case "$2" in
            build)
                pushd build
                xargo build --target "$TARGET" $CARGOFLAGS
                popd
                ;;
            clean)
                pushd build
                xargo clean
                popd
                ;;
            fetch)
                git clone --recursive "$GIT" build
                ;;
            unfetch)
                rm -rf build
                ;;
            update)
                pushd build
                xargo update
                popd
                ;;
            *)
                echo "$0 {package} {build|clean|fetch|update}"
                ;;
        esac
        popd
    else
        echo "$0: recipe '$1' not found"
    fi
else
    echo "$0 {package} {build|clean|fetch|update}"
fi
