VERSION=2.7.0
TAR=https://duktape.org/duktape-$VERSION.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sed -i "s/= gcc/= $TARGET-gcc/g" Makefile.cmdline
    "$REDOX_MAKE" -f Makefile.cmdline -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/bin"
    cp ./duk "$1/bin/duk"

    skip=1
}
