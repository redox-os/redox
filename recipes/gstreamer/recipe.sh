VERSION=1.14.4
TAR=https://gstreamer.freedesktop.org/src/gstreamer/gstreamer-$VERSION.tar.xz
BUILD_DEPENDS=(gettext glib libffi libiconv pcre zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    export GLIB_GENMARSHAL="$(which glib-genmarshal)"
    export GLIB_MKENUMS="$(which glib-mkenums)"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static \
        --disable-benchmarks \
        --disable-examples \
        --disable-tests
    make -j"$(nproc)" V=1
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
