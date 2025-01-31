VERSION=3.10
TAR=https://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-$VERSION.tar.gz
BUILD_DEPENDS=(curl expat libjpeg libpng nghttp2 openssl1 sdl1 zlib freetype2 liborbital libiconv)
DEPENDS="ca-certificates orbital"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export TARGET="framebuffer"
    export CFLAGS="-I${COOKBOOK_SYSROOT}/include -I${PWD}/inst-${TARGET}/include"
    export LDFLAGS="-L${COOKBOOK_SYSROOT}/lib -L${PWD}/inst-${TARGET}/lib -Wl,--allow-multiple-definition"
    # nghttp2 is not linked for some reason
    export LDFLAGS="${LDFLAGS} -lcurl -lnghttp2"
    "$REDOX_MAKE" PREFIX=/usr V=1 -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath "$1")"
    "$REDOX_MAKE" DESTDIR="$dest" PREFIX=/usr install
    mkdir -pv "$dest/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$dest/ui/apps/00_netsurf"
    skip=1
}
