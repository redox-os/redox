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
        cd "recipes/$1"
        source recipe.sh
        for arg in "${@:2}"
        do
            case "$arg" in
                build)
                    cd build
                    xargo build --target "$TARGET" $CARGOFLAGS
                    ;;
                clean)
                    cd build
                    xargo clean
                    ;;
                fetch)
                    git clone --recursive "$GIT" build
                    ;;
                stage)
                    mkdir -p stage/bin
                    cd build
                    #TODO xargo install --root "../stage" $CARGOFLAGS
                    cp -v $(find target/x86_64-unknown-redox/debug/ -maxdepth 1 -type f ! -name "*.*") ../stage/bin
                    ;;
                unstage)
                    rm -rf stage
                    ;;
                unfetch)
                    rm -rf build
                    ;;
                update)
                    cd build
                    xargo update
                    ;;
                *)
                    echo "$0 {package} {build|clean|fetch|update}"
                    ;;
            esac
        done
    else
        echo "$0: recipe '$1' not found"
    fi
else
    echo "$0 {package} {build|clean|fetch|update}"
fi
