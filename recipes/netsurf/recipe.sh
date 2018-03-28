VERSION=3.7
TAR=http://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-$VERSION.tar.gz
BUILD_DEPENDS=(curl expat libjpeg libpng openssl sdl zlib freetype)
DEPENDS="ca-certificates orbital"

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
    export TARGET="framebuffer"
    export CFLAGS="-I$sysroot/include -I${PWD}/inst-${TARGET}/include"
    export LDFLAGS="-L$sysroot/lib -L${PWD}/inst-${TARGET}/lib"

    make V=1
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
