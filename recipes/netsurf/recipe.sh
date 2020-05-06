VERSION=3.7
TAR=http://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-$VERSION.tar.gz
BUILD_DEPENDS=(curl expat libjpeg libpng nghttp2 openssl sdl zlib freetype liborbital libiconv)
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
    sysroot="$(realpath ../sysroot)"
    export TARGET="framebuffer"
    export CFLAGS="-I$sysroot/include -I${PWD}/inst-${TARGET}/include"
    export LDFLAGS="-L$sysroot/lib -L${PWD}/inst-${TARGET}/lib -static -Wl,--allow-multiple-definition -Wl,-Bstatic"
    # nghttp2 is not linked for some reason
    export LDFLAGS="${LDFLAGS} -lcurl -lnghttp2"
    "$REDOX_MAKE" V=1 -j"$($NPROC)"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    mkdir -pv "$dest/ui/apps"
    cp -v ../manifest "$dest/ui/apps/00_netsurf"
    skip=1
}
