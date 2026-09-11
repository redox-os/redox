#!/usr/bin/env bash

set -e

# This script generates a "redox.patch" for a tar-based recipe

if [ -z "$1" ]
then
    echo "Generates a 'redox.patch' for a tar-based recipe"
    echo ""
    echo "Usage: $0 <recipe-name>"
    exit 1
fi

RECIPE_PATH=$(make find.$1)
RECIPE_PATH=${RECIPE_PATH//$'\r'/}

if [ ! -d "$RECIPE_PATH/source" ]; then
    echo "Error: 'source' directory not found"
    exit 1
fi

if [ ! -f "$RECIPE_PATH/source.tar" ]; then
    echo "Error: 'source.tar' not found"
    exit 1
fi

export TZ=UTC
set -x

cd $RECIPE_PATH
mv source source-new
mkdir source
tar xf source.tar -C source --strip-components=1
diff -ruwN source source-new > redox.patch || true
rm -rf source
mv source-new source
