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
        for arg in "${@:2}"
        do
            case "$arg" in
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
                install)
                    mkdir -p root/bin
                    pushd build
                    #TODO xargo install --root "../root" $CARGOFLAGS
                    cp -v $(find target/x86_64-unknown-redox/debug/ -maxdepth 1 -type f ! -name "*.*") ../root/bin
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
        done
        popd
    else
        echo "$0: recipe '$1' not found"
    fi
else
    echo "$0 {package} {build|clean|fetch|update}"
fi
