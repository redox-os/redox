VERSION=4.2.1
TAR=https://ftp.gnu.org/gnu/make/make-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export CFLAGS="-DPOSIX -DNO_ARCHIVES -DNO_OUTPUT_SYNC"
    export LDFLAGS="-static"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --without-guile
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    skip=1
}
