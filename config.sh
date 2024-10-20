#!/usr/bin/env bash
set -e

# Configuration
if [ -z "${TARGET}" ]
then
    export TARGET=x86_64-unknown-redox
fi
ARCH="${TARGET%%-*}"
HOST="$TARGET"
if [ x"${HOST}" == x"riscv64gc-unknown-redox" ] ; then
	HOST="riscv64-unknown-redox"
fi

# Automatic variables
ROOT="$(cd `dirname "$0"` && pwd)"
REPO="$ROOT/repo/$TARGET"
export PATH="${ROOT}/bin:$PATH"

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

BUILD="$(cc -dumpmachine)"

export PKG_CONFIG_FOR_BUILD="pkg-config"

if [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "FreeBSD" ]]; then
    # GNU find
    FIND="gfind";

    # GNU stat from Homebrew or MacPorts
    if [ ! -z "$(which brew)" ]; then
        STAT="$(brew --prefix)/opt/coreutils/libexec/gnubin/stat";
    elif [ ! -z "$(which port)" ]; then
        # TODO: find a programatic way of asking MacPorts for it's root dir.
        STAT="/opt/local/opt/coreutils/libexec/gnubin/stat";
    elif [ ! -z "$(which pkg)" ]; then
        STAT="gnustat"
    else
        echo "Please install either Homebrew or MacPorts and run the boostrap script."
        exit 1
    fi
else
    FIND="find"
    STAT="stat";
fi

export FIND
export STAT

if [ ! "$(uname -s)" = "Redox" ]
then
function docgen {
    "$ROOT/docgen/target/release/docgen" "$@"
}

function pkgar {
    "$ROOT/pkgar/target/release/pkgar" "$@"
}
fi
