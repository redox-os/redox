VERSION=20110206
TAR=https://www.etalabs.net/releases/libc-bench-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -v "$dest/bin"
    cp -v "libc-bench" "$dest/bin"
    skip=1
}
