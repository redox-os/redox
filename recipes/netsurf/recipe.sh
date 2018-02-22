VERSION=3.7
TAR=http://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-$VERSION.tar.gz
BUILD_DEPENDS=(expat curl sdl openssl zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export AR="${HOST}-ar"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    export TARGET="framebuffer"
    export PKG_CONFIG_PATH="$PWD/../sysroot/lib/pkgconfig"

    make
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
    skip=1
}

