#!/usr/bin/env bash
set -e

# Configuration
if [ -z "${TARGET}" ]
then
    export TARGET=x86_64-unknown-redox
fi
if [ $(uname -s) = 'Redox' ]
then
    export IS_REDOX="1"
fi

ARCH="${TARGET%%-*}"
HOST="$TARGET"
if [ x"${HOST}" == x"riscv64gc-unknown-redox" ] ; then
	HOST="riscv64-unknown-redox"
fi

# Automatic variables
ROOT="$(cd `dirname "$0"` && pwd)"

export AR="${HOST}-gcc-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-ld"
export NM="${HOST}-gcc-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export PKG_CONFIG="${HOST}-pkg-config"
export RANLIB="${HOST}-gcc-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

if [ -n "${CC_WRAPPER}" ]
then
    export CC="${CC_WRAPPER} ${CC}"
    export CXX="${CC_WRAPPER} ${CXX}"
fi

BUILD="$(cc -dumpmachine)"

export PKG_CONFIG_FOR_BUILD="pkg-config"

if [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "FreeBSD" ]]; then
    # GNU find
    FIND="gfind";
else
    FIND="find";
fi

export FIND

if [ -z "${IS_REDOX}" ]
then
function pkgar {
    "$ROOT/pkgar/target/release/pkgar" "$@"
}
function cook {
    "$ROOT/target/release/cook" "$@"
}
function repo {
    "$ROOT/target/release/repo" "$@"
}
function repo_builder {
    "$ROOT/target/release/repo_builder" "$@"
}
function list_recipes {
    "$ROOT/target/release/list_recipes" "$@"
}
function find_recipe {
    "$ROOT/target/release/find_recipe" "$@"
}
fi
