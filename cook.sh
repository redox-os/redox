#!/usr/bin/env bash
set -e
shopt -s nullglob

source config.sh

# Variables to be overriden by recipes
export BINDIR=bin
export CARGO=(env RUSTFLAGS="$PREFIX_RUSTFLAGS -C link-arg=-zmuldefs" cargo)
export CARGOBUILD=rustc
export CARGOFLAGS=
export DEBUG=
export EXAMPLES=
export PREPARE_COPY=1

if hash sha256sum 2>/dev/null
then
    SHASUM="sha256sum"
else
    SHASUM="shasum -a 256"
fi

if [ ! "$(uname -s)" = "Redox" ]
then
function docgen {
    CC=cc AR=ar RANLIB=ranlib cargo run --release --manifest-path "$ROOT/docgen/Cargo.toml" --bin docgen -- "$@"
}

function pkg {
    CC=cc AR=ar RANLIB=ranlib cargo run --release --manifest-path "$ROOT/pkgutils/Cargo.toml" --bin pkg -- "$@"
}

function pkgar {
    CC=cc AR=ar RANLIB=ranlib cargo run --release --manifest-path "$ROOT/pkgar/Cargo.toml" --bin pkgar -- "$@"
}
fi

function usage {
    echo "cook.sh $1 <op>" >&2
    echo "  dist" >&2
    echo "  distclean" >&2
    echo "  build" >&2
    echo "  clean" >&2
    echo "  fetch" >&2
    echo "  unfetch" >&2
    echo "  pkg" >&2
    echo "  unpkg" >&2
    echo "  prepare" >&2
    echo "  unprepare" >&2
    echo "  publish" >&2
    echo "  unpublish" >&2
    echo "  stage" >&2
    echo "  unstage" >&2
    echo "  tar" >&2
    echo "  untar" >&2
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
            op $1 pkg
            ;;
        distclean)
            op $1 unpkg
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

                    if [ -n "$TAR_SHA256" ]
                    then
                        $SHASUM -c <<< "${TAR_SHA256} source.tar"
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

                    ORIGIN_BRANCH="$(git branch --remotes | grep '^  origin/HEAD -> ' | cut -d ' ' -f 5-)"
                    if [ -n "$BRANCH" ]
                    then
                        ORIGIN_BRANCH="origin/$BRANCH"
                    fi

                    if [ "$(git rev-parse HEAD)" != "$(git rev-parse $ORIGIN_BRANCH)" ]
                    then
                        git checkout -B "$(echo "$ORIGIN_BRANCH" | cut -d / -f 2-)" "$ORIGIN_BRANCH"
                    fi
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
        prepare)
            skip=0
            if [ "$(type -t recipe_prepare)" = "function" ]
            then
                recipe_prepare
            fi
            if [ "$skip" -eq "0" ]
            then
                rm -rf "${COOKBOOK_SYSROOT}"
                mkdir "${COOKBOOK_SYSROOT}"

                if [ ${#BUILD_DEPENDS} -gt 0 ]
                then
                    pushd $ROOT
                        ./repo.sh "${BUILD_DEPENDS[@]}"
                    popd

                    for i in "${BUILD_DEPENDS[@]}"
                    do
                        pkgar \
                            extract \
                            "${COOKBOOK_SYSROOT}" \
                            --archive "$REPO/$i.pkgar" \
                            --pkey "${ROOT}/build/id_ed25519.pub.toml"
                    done
                fi

                rm -rf "${COOKBOOK_BUILD}"
                if [ "$PREPARE_COPY" -eq "0" ]
                then
                    mkdir "${COOKBOOK_BUILD}"
                else
                    cp -Rp source "${COOKBOOK_BUILD}"
                fi

                for patch in *.patch
                do
                    patch -p1 -d "${COOKBOOK_BUILD}" < "$patch"
                done
            fi
            ;;
        unprepare)
            rm -rf "${COOKBOOK_BUILD}"
            rm -rf "${COOKBOOK_SYSROOT}"
            ;;
        version)
            pushd "${COOKBOOK_BUILD}" > /dev/null
            skip=0
            if [ "$(type -t recipe_version)" = "function" ]
            then
                recipe_version
            fi
            if [ "$skip" -eq "0" ]
            then
                # there's an unstable built-in cargo config command, so hack around it
                cargo-config config package.version | tr -d '"'
            fi
            popd > /dev/null
            ;;
        gitversion)
            if [ -d "${COOKBOOK_BUILD}"/.git ]
            then
                echo "$(op $1 version)-$(git -C "${COOKBOOK_BUILD}" rev-parse --short HEAD)"
            else
                op $1 version
            fi
            ;;
        build)
            pushd "${COOKBOOK_BUILD}" > /dev/null
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

            if [ -n "$CARGO_PACKAGE" ]; then
                package_flag="--package=$CARGO_PACKAGE"
            else
                package_flag=
            fi

            if [ "$skip" -eq "0" ]
            then
                "${CARGO[@]}" "$CARGOBUILD" --target "$TARGET" $release_flag $package_flag $CARGOFLAGS
            fi
            popd > /dev/null
            ;;
        clean)
            pushd "${COOKBOOK_BUILD}" > /dev/null
            skip=0
            if [ "$(type -t recipe_clean)" = "function" ]
            then
                recipe_clean
            fi
            if [ "$skip" -eq "0" ]
            then
                "${CARGO[@]}" clean
            fi
            popd > /dev/null
            ;;
        stage)
            op $1 unstage
            mkdir -p "${COOKBOOK_STAGE}"
            stage="$(realpath "${COOKBOOK_STAGE}")"
            source="$(realpath source)"
            pushd "${COOKBOOK_BUILD}" > /dev/null
            skip=0
            if [ "$(type -t recipe_stage)" = "function" ]
            then
                recipe_stage "$stage"
            fi
            if [ "$skip" -eq "0" ]
            then
                #TODO "${CARGO[@]}" install --root "$stage" $CARGOFLAGS
                if [ "$DEBUG" == 1 ]
                then
                    build=debug
                else
                    build=release
                fi

                bins="$(find target/$TARGET/$build/ -maxdepth 1 -type f ! -name '*.*')"
                if [ -z "$bins" ] || [ "$EXAMPLES" == 1 ]
                then
                    example=true
                    bins="$bins $(find target/$TARGET/$build/examples/ -maxdepth 1 -type f ! -name '*.*' ! -name '*-*' \
                            2> /dev/null || true)"
                fi
                if [ -n "$bins" ]
                then
                    if [ -n "$example" ] && [ "$EXAMPLES" != 1 ]
                    then
                        echo "$(tput bold)Note$(tput sgr0): No binaries detected, using example binaries"
                    fi
                    mkdir -p "$stage/$BINDIR"
                    for bin in $bins
                    do
                        if [ "$DEBUG" == 1 ]
                        then
                            cp -v "$bin" "$stage/$BINDIR/$(basename $bin)"
                        else
                            "${STRIP}" -v "$bin" -o "$stage/$BINDIR/$(basename $bin)"
                        fi
                    done
                else
                    echo "$(tput bold)Warning$(tput sgr0): Recipe does not have any binaries" >&2
                fi

                docgen "$source" "$stage/ref"
            fi
            popd > /dev/null
            ;;
        unstage)
            rm -rfv "${COOKBOOK_STAGE}"
            ;;
        pkg)
            pkgar \
                create \
                --archive "${COOKBOOK_STAGE}.pkgar" \
                --skey "${ROOT}/build/id_ed25519.toml" \
                "${COOKBOOK_STAGE}"
            ;;
        unpkg)
            rm -fv "${COOKBOOK_STAGE}.pkgar"
            ;;
        tar)
            echo "name = \"$1\"" > "${COOKBOOK_STAGE}.toml"
            echo "version = \"$(op $1 version)\"" >> "${COOKBOOK_STAGE}.toml"
            echo "target = \"$TARGET\"" >> "${COOKBOOK_STAGE}.toml"

            # Add runtime dependencies to package if they exist
            if [ -n "$DEPENDS" ]
            then
                # Remove leading and trailing whitespace, replace whitespace between
                # package names with commas, and surround package names with quotes
                dependencies=$(echo -e "$DEPENDS" | sed -E 's/^[[:space:]]*//;s/[[:space:]]*$//;s/[[:space:]]+/,/g;s/[^, ][^, ]*/"&"/g')
                echo "depends = [$dependencies]" >> "${COOKBOOK_STAGE}.toml"
            else
                echo "depends = []" >> "${COOKBOOK_STAGE}.toml"
            fi

            rm -rf "${COOKBOOK_STAGE}/pkg"
            mkdir -p "${COOKBOOK_STAGE}/pkg"

            pushd "${COOKBOOK_STAGE}" > /dev/null
            find -L . -type f | cut -d / -f 2- | sort | while read file
            do
                $SHASUM "$file" >> "pkg/$1.sha256sums"
            done
            popd > /dev/null

            cp -v "${COOKBOOK_STAGE}.toml" "${COOKBOOK_STAGE}/pkg/$1.toml"
            pushd "$(dirname "${COOKBOOK_STAGE}")" > /dev/null
                pkg --target="$TARGET" create "$(basename "${COOKBOOK_STAGE}")"
            popd > /dev/null
            ;;
        untar)
            rm -rfv "${COOKBOOK_STAGE}.tar.gz" "${COOKBOOK_STAGE}.sig" "${COOKBOOK_STAGE}.toml"
            ;;
        publish)
            mkdir -p "$REPO"
            cp -v "${COOKBOOK_STAGE}.tar.gz" "$REPO/$1.tar.gz"
            cp -v "${COOKBOOK_STAGE}.sig" "$REPO/$1.sig"
            cp -v "${COOKBOOK_STAGE}.toml" "$REPO/$1.toml"
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
    recipe_path=`target/release/find_recipe $1`
    if [ -d "$ROOT/$recipe_path" ]
    then
        export COOKBOOK_RECIPE="${ROOT}/$recipe_path"

        TARGET_DIR="${COOKBOOK_RECIPE}/target/${TARGET}"
        mkdir -p "${TARGET_DIR}"

        export COOKBOOK_BUILD="${TARGET_DIR}/build"
        export COOKBOOK_STAGE="${TARGET_DIR}/stage"
        export COOKBOOK_SOURCE="${COOKBOOK_RECIPE}/source"
        export COOKBOOK_SYSROOT="${TARGET_DIR}/sysroot"

        export PKG_CONFIG_ALLOW_CROSS=1
        export PKG_CONFIG_PATH=
        export PKG_CONFIG_LIBDIR="${COOKBOOK_SYSROOT}/lib/pkgconfig"
        export PKG_CONFIG_SYSROOT_DIR="${COOKBOOK_SYSROOT}"

        cd "${COOKBOOK_RECIPE}"

        if [ -e recipe.sh ]; then
            source recipe.sh
        fi

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
