#!/bin/bash

export RUST_TARGET_PATH="$PWD/targets"
export RUSTFLAGS="--cfg redox"
export CARGOFLAGS=
TARGET=x86_64-unknown-redox

set -e

function op {
    echo "$1" "$2"
    case "$2" in
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
            rm -rf build
            ;;
        update)
            pushd build > /dev/null
            xargo update
            popd > /dev/null
            ;;
        build)
            pushd build > /dev/null
            xargo build --target "$TARGET" $CARGOFLAGS
            popd > /dev/null
            ;;
        clean)
            pushd build > /dev/null
            xargo clean
            popd > /dev/null
            ;;
        stage)
            mkdir -p stage/bin
            pushd build > /dev/null
            #TODO xargo install --root "../stage" $CARGOFLAGS
            cp -v $(find target/x86_64-unknown-redox/debug/ -maxdepth 1 -type f ! -name "*.*") ../stage/bin
            popd > /dev/null
            ;;
        unstage)
            rm -rf stage
            ;;
        tar)
            pushd stage > /dev/null
            tar cf ../stage.tar .
            popd > /dev/null
            ;;
        untar)
            rm -rf stage.tar
            ;;
        *)
            echo "$0 {package} {build|clean|fetch|update}"
            ;;
    esac
}

if [ -n "$1" ]
then
    if [ -d "recipes/$1" ]
    then
        cd "recipes/$1"
        source recipe.sh
        for arg in "${@:2}"
        do
            op "$1" "$arg"
        done
    else
        echo "$0: recipe '$1' not found"
    fi
else
    echo "$0 {package} {build|clean|fetch|unfetch|stage|unstage|tar|untar|update}"
fi
