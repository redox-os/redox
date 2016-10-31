#!/bin/bash

export RUST_TARGET_PATH="$PWD/targets"
export RUSTFLAGS="--cfg redox"
export CARGOFLAGS=
TARGET=x86_64-unknown-redox
REPO="$PWD/repo/$TARGET"

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
            rm -rfv build
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
            strip -v stage/bin/*
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
            echo "cook.sh $1 {build|clean|fetch|unfetch|publish|unpublish|stage|unstage|tar|untar|update}"
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
        echo "cook.sh: recipe '$1' not found"
    fi
else
    echo "cook.sh {package} {build|clean|fetch|unfetch|publish|unpublish|stage|unstage|tar|untar|update}"
fi
