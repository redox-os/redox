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

# Cookbook requires correct CWD to work
cd `dirname "$0"`

# Automatic variables
ROOT=`pwd`

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
function repo {
    "$ROOT/target/release/repo" "$@"
}
fi
