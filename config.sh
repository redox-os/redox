#!/usr/bin/env bash
set -e

# Configuration
export TARGET=x86_64-unknown-redox

# Automatic variables
ROOT="$(cd `dirname "$0"` && pwd)"
REPO="$ROOT/repo/$TARGET"
export CC="x86_64-elf-redox-gcc"
export XARGO_HOME="$ROOT/xargo"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # GNU find
    FIND="gfind";

    # GNU stat from Homebrew or MacPorts
    if [ ! -z "$(which brew)" ]; then
        STAT="$(brew --prefix)/opt/coreutils/libexec/gnubin/stat";
    elif [ ! -z "$(which port)" ]; then
        # TODO: find a programatic way of asking MacPorts for it's root dir.
        STAT="/opt/local/opt/coreutils/libexec/gnubin/stat";
    else
        echo "Please install either Homebrew or MacPorts and run the boostrap script."
        exit 1
    fi
else
    FIND="find"
    STAT="stat";
fi
