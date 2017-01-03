#!/bin/bash

ROOT="$PWD"
export RUST_TARGET_PATH="$ROOT/targets"
export CARGOFLAGS=--verbose
export CFLAGS="-static -nostartfiles -nostdlib -nodefaultlibs \
    -undef -imacros $ROOT/libc-artifacts/define.h \
    -isystem $ROOT/libc-artifacts/usr/include \
    -L $ROOT/libc-artifacts/usr/lib \
    $ROOT/libc-artifacts/usr/lib/crt0.o -lm -lc -lgcc \
    -fno-stack-protector -U_FORTIFY_SOURCE"
export CARGO_BUILD_RUSTFLAGS="--verbose -Z print-link-args -C linker=gcc -C link-args=\"\$CFLAGS\""
export TARGET=x86_64-unknown-redox
REPO="$ROOT/repo/$TARGET"

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
        update)
            pushd build > /dev/null
            xargo update
            popd > /dev/null
            ;;
        build)
            pushd build > /dev/null
            cp -r "$ROOT/Xargo.toml" "$ROOT/libc-artifacts" .
            xargo build --target "$TARGET" --release $CARGOFLAGS
            popd > /dev/null
            ;;
        test)
            pushd build > /dev/null
            cp -r "$ROOT/Xargo.toml" "$ROOT/libc-artifacts" .
            xargo test --no-run --target "$TARGET" --release $CARGOFLAGS
            popd > /dev/null
            ;;
        clean)
            pushd build > /dev/null
            xargo clean
            popd > /dev/null
            ;;
        stage)
            mkdir -p stage
            pushd build > /dev/null
            if [ "$(type -t recipe_stage)" = "function" ]
            then
                recipe_stage ../stage
            else
                #TODO xargo install --root "../stage" $CARGOFLAGS
                bins="$(find target/$TARGET/release/ -maxdepth 1 -type f ! -name '*.*')"
                if [ -n "$bins" ]
                then
                    mkdir -p ../stage/bin
                    for bin in $bins
                    do
                        cp -v "$bin" "../stage/bin/$(basename $bin)"
                        strip -v "../stage/bin/$(basename $bin)"
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
    echo "cook.sh {package} {dist|distclean|build|clean|fetch|unfetch|publish|unpublish|stage|unstage|tar|untar|update}"
fi
