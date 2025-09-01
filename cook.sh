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

function usage {
    echo "cook.sh $1 <op>" >&2
    echo "  distclean" >&2
    echo "  unfetch" >&2
}

function op {
    if [ ! "$COOK_QUIET" = "1" ]
    then
        echo -e "\033[01;38;5;215mcook - $1 $2\033[0m" >&2
    fi

    case "$2" in
        distclean)
            op $1 unpkg
            op $1 unstage
            op $1 unprepare
            ;;
        unfetch)
            rm -rfv source source.tar
            ;;
        unprepare)
            rm -rf "${COOKBOOK_BUILD}"
            rm -rf "${COOKBOOK_SYSROOT}"
            ;;
        unstage)
            rm -rfv "${COOKBOOK_STAGE}"
            rm -fv "${TARGET_DIR}/auto_deps.toml"
            ;;
        unpkg)
            rm -fv "${COOKBOOK_STAGE}.pkgar" "${COOKBOOK_STAGE}.toml"
            ;;
        *)
            usage $1
            ;;
    esac
}

if [ -n "$1" ]
then
    if (echo "$1" | grep '.*/.*' >/dev/null); then
        recipe_name=$(basename "$1")
        recipe_path="$1"
    else
        recipe_name="$1"
        recipe_path=`target/release/find_recipe $recipe_name`
    fi

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
            op "$recipe_name" "$i"
        done
    elif [ "$IGNORE_ERROR" != "1" ]
    then
        echo "cook.sh: recipe '$recipe_name' at not found" >&2
        exit 1
    fi
else
    usage "{package}"
fi
