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

function usage {
    echo "cook.sh $1 <op>" >&2
    echo "  dist" >&2
    echo "  distclean" >&2
    echo "  build" >&2
    echo "  clean" >&2
    echo "  fetch" >&2
    echo "  unfetch" >&2
    echo "  publish" >&2
    echo "  unpublish" >&2
    echo "  stage" >&2
    echo "  unstage" >&2
    echo "  tar" >&2
    echo "  untar" >&2
    echo "  update" >&2
    echo "  version" >&2
}

function op {
    if [ ! "$COOK_QUIET" = "1" ]
    then
        echo -e "\033[01;38;5;215mcook - $1 $2\033[0m" >&2
    fi

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
            if [ -n "$TAR" ]
            then
                if [ ! -f source.tar ]
                then
                    wget "$TAR" -O source.tar
                fi

                if [ ! -d source ]
                then
                    mkdir source
                    tar xvf source.tar -C source --strip-components 1
                fi

                rm -rf build
                cp -r source build
            elif [ -n "$GIT" ]
            then
                if [ ! -d source ]
                then
                    git clone --recursive "$GIT" source
                fi

                pushd source > /dev/null
                git pull
                git submodule sync
                git submodule update --init --recursive
                popd > /dev/null

                rm -rf build
                cp -r source build
            fi

            ;;
        unfetch)
            rm -rfv build source
            if [ -n "$TAR" ]
            then
                rm -f source.tar
            fi
            ;;
        version)
            pushd build > /dev/null
            skip="0"
            if [ "$(type -t recipe_version)" = "function" ]
            then
                recipe_version || skip="1"
            fi
            if [ "$skip" -eq "0" ]
            then
                cargo config package.version | tr -d '"'
            fi
            popd > /dev/null
            ;;
        gitversion)
            if [ -d build/.git ]
            then
                echo "$(op $1 version)-$(git -C build rev-parse --short HEAD)"
            else
                op $1 version
            fi
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
            echo "name = \"$1\"" > "stage.toml"
            echo "version = \"$(op $1 version)\"" >> "stage.toml"
            echo "target = \"$TARGET\"" >> "stage.toml"
            mkdir -p stage/pkg
            cp -v stage.toml "stage/pkg/$1.toml"
            $ROOT/pkgutils/target/release/pkg create stage
            ;;
        untar)
            rm -rfv stage.tar stage.sig stage.toml
            ;;
        publish)
            mkdir -p "$REPO"
            cp -v stage.tar "$REPO/$1.tar"
            cp -v stage.sig "$REPO/$1.sig"
            cp -v stage.toml "$REPO/$1.toml"
            ;;
        unpublish)
            rm -rfv "$REPO/$1.tar" "$REPO/$1.sig" "$REPO/$1.toml"
            ;;
        *)
            usage $1
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
        echo "cook.sh: recipe '$1' not found" >&2
        exit 1
    fi
else
    usage "{package}"
fi
