#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

# Variables to be overriden by recipes
export BINDIR=bin
export CARGOFLAGS=
export DEBUG=

if [ ! "$(uname -s)" = "Redox" ]
then
function pkg {
    CC=cc cargo run --release --manifest-path "$ROOT/pkgutils/Cargo.toml" --bin pkg -- $@
}
fi

function usage {
    echo "cook.sh $1 <op>" >&2
    echo "  dist" >&2
    echo "  distclean" >&2
    echo "  build" >&2
    echo "  clean" >&2
    echo "  diff" >&2
    echo "  diff_origin" >&2
    echo "  diff_upstream" >&2
    echo "  difftool" >&2
    echo "  difftool_origin" >&2
    echo "  difftool_upstream" >&2
    echo "  fetch" >&2
    echo "  unfetch" >&2
    echo "  prepare" >&2
    echo "  unprepare" >&2
    echo "  publish" >&2
    echo "  unpublish" >&2
    echo "  stage" >&2
    echo "  unstage" >&2
    echo "  status" >&2
    echo "  status_origin" >&2
    echo "  status_upstream" >&2
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
            op $1 prepare
            op $1 build
            op $1 stage
            op $1 tar
            ;;
        distclean)
            op $1 untar
            op $1 unstage
            op $1 unprepare
            ;;
        fetch)
            skip=0
            if [ "$(type -t recipe_fetch)" = "function" ]
            then
                recipe_fetch
            fi
            if [ "$skip" -eq "0" ]
            then
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
                elif [ -n "$GIT" ]
                then
                    if [ ! -d source ]
                    then
                        if [ -n "$BRANCH" ]
                        then
                            git clone --recursive "$GIT" -b "$BRANCH" source
                        else
                            git clone --recursive "$GIT" source
                        fi
                    fi

                    pushd source > /dev/null
                    git remote set-url origin "$GIT"
                    git fetch origin
                    if [ -n "$GIT_UPSTREAM" ]
                    then
                        git remote set-url upstream "$GIT_UPSTREAM" &> /dev/null ||
                        git remote add upstream "$GIT_UPSTREAM"
                        git fetch upstream
                    fi
                    git pull
                    git submodule sync --recursive
                    git submodule update --init --recursive
                    popd > /dev/null
                fi
            fi
            ;;
        unfetch)
            rm -rfv source
            if [ -n "$TAR" ]
            then
                rm -f source.tar
            fi
            ;;
        status)
            if [ -n "$TAR" ]
            then
                tar --compare --file="source.tar" -C "source" --strip-components=1 2>&1 |
                grep -v "tar: :" | grep -v '\(Mod time\|Mode\|Gid\|Uid\) differs' ||
                true
            elif [ -n "$GIT" ]
            then
                git -C source diff --stat --color
            fi
            ;;
        status_origin)
            if [ -n "$GIT" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source diff --stat --color "origin/$BRANCH"
                else
                    git -C source diff --stat --color "origin/master"
                fi
            fi
            ;;
        status_upstream)
            if [ -n "$GIT_UPSTREAM" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source diff --stat --color "upstream/$BRANCH"
                else
                    git -C source diff --stat --color "upstream/master"
                fi
            fi
            ;;
        diff)
            if [ -n "$GIT" ]
            then
                git -C source diff
            fi
            ;;
        diff_origin)
            if [ -n "$GIT" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source diff "origin/$BRANCH"
                else
                    git -C source diff "origin/master"
                fi
            fi
            ;;
        diff_upstream)
            if [ -n "$GIT_UPSTREAM" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source diff "upstream/$BRANCH"
                else
                    git -C source diff "upstream/master"
                fi
            fi
            ;;
        difftool)
            if [ -n "$GIT" ]
            then
                git -C source difftool -d
            fi
            ;;
        difftool_origin)
            if [ -n "$GIT" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source difftool -d "origin/$BRANCH"
                else
                    git -C source difftool -d "origin/master"
                fi
            fi
            ;;
        difftool_upstream)
            if [ -n "$GIT_UPSTREAM" ]
            then
                if [ -n "$BRANCH" ]
                then
                    git -C source difftool -d "upstream/$BRANCH"
                else
                    git -C source difftool -d "upstream/master"
                fi
            fi
            ;;
        update)
            pushd source > /dev/null
            skip=0
            if [ "$(type -t recipe_update)" = "function" ]
            then
                recipe_update
            fi
            if [ "$skip" -eq "0" ]
            then
                xargo update
            fi
            popd > /dev/null
            ;;
        prepare)
            rm -rf sysroot
            mkdir sysroot

            if [ ${#BUILD_DEPENDS} -gt 0 ]
            then
                pushd $ROOT
                    ./repo.sh "${BUILD_DEPENDS[@]}"
                popd

                for i in "${BUILD_DEPENDS[@]}"
                do
                    pkg --target=$TARGET install --root sysroot "$REPO/$i.tar.gz"
                done
            fi

            rm -rf build
            cp -rp source build

            for patch in *.patch
            do
                patch -p1 -d build < "$patch"
            done
            ;;
        unprepare)
            rm -rf build
            rm -rf sysroot
            ;;
        version)
            pushd build > /dev/null
            skip=0
            if [ "$(type -t recipe_version)" = "function" ]
            then
                recipe_version
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
        build)
            pushd build > /dev/null
            skip=0
            if [ "$(type -t recipe_build)" = "function" ]
            then
                recipe_build
            fi

            release_flag="--release"
            if [ "$DEBUG" == 1 ]
            then
                release_flag=
            fi

            if [ "$skip" -eq "0" ]
            then
                cp -r "$ROOT/Xargo.toml" .
                xargo build --target "$TARGET" $release_flag $CARGOFLAGS
            fi
            popd > /dev/null
            ;;
        test)
            pushd build > /dev/null
            skip=0
            if [ "$(type -t recipe_test)" = "function" ]
            then
                recipe_test
            fi

            release_flag="--release"
            if [ "$DEBUG" == 1 ]
            then
                release_flag=
            fi

            if [ "$skip" -eq "0" ]
            then
                cp -r "$ROOT/Xargo.toml" .
                xargo test --no-run --target "$TARGET" $release_flag $CARGOFLAGS
            fi
            popd > /dev/null
            ;;
        clean)
            pushd build > /dev/null
            skip=0
            if [ "$(type -t recipe_clean)" = "function" ]
            then
                recipe_clean
            fi
            if [ "$skip" -eq "0" ]
            then
                xargo clean
            fi
            popd > /dev/null
            ;;
        stage)
            op $1 unstage
            mkdir -p stage
            pushd build > /dev/null
            skip=0
            if [ "$(type -t recipe_stage)" = "function" ]
            then
                recipe_stage ../stage
            fi
            if [ "$skip" -eq "0" ]
            then
                #TODO xargo install --root "../stage" $CARGOFLAGS
                if [ "$DEBUG" == 1 ]
                then
                    build=debug
                else
                    build=release
                fi
                bins="$(find target/$TARGET/$build/ -maxdepth 1 -type f ! -name '*.*')"
                if [ -n "$bins" ]
                then
                    mkdir -p "../stage/$BINDIR"
                    for bin in $bins
                    do
                        if [ "$DEBUG" == 1 ]
                        then
                            cp -v "$bin" "../stage/$BINDIR/$(basename $bin)"
                        else
                            strip -v "$bin" -o "../stage/$BINDIR/$(basename $bin)"
                        fi
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

            # Add runtime dependencies to package if they exist
            if [ -n "$DEPENDS" ]
            then
                # Remove leading and trailing whitespace, replace whitespace between
                # package names with commas, and surround package names with quotes
                dependencies=$(echo -e "$DEPENDS" | sed -E 's/^[[:space:]]*//;s/[[:space:]]*$//;s/[[:space:]]+/,/g;s/[^, ][^, ]*/"&"/g')
                echo "depends = [$dependencies]" >> "stage.toml"
			else
				echo "depends = []" >> "stage.toml"
            fi

            mkdir -p stage/pkg
            cp -v stage.toml "stage/pkg/$1.toml"
            pkg --target=$TARGET create stage
            ;;
        untar)
            rm -rfv stage.tar.gz stage.sig stage.toml
            ;;
        publish)
            mkdir -p "$REPO"
            cp -v stage.tar.gz "$REPO/$1.tar.gz"
            cp -v stage.sig "$REPO/$1.sig"
            cp -v stage.toml "$REPO/$1.toml"
            ;;
        unpublish)
            rm -rfv "$REPO/$1.tar.gz" "$REPO/$1.sig" "$REPO/$1.toml"
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

        ops=()
        for arg in "${@:2}"
        do
            if [ "$arg" == "--debug" ]
            then
                DEBUG=1
            else
                ops[${#ops[@]}]="$arg"
            fi
        done

        for i in "${ops[@]}"
        do
            op "$1" "$i"
        done
    else
        echo "cook.sh: recipe '$1' not found" >&2
        exit 1
    fi
else
    usage "{package}"
fi
