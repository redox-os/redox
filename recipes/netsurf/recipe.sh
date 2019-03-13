VERSION=3.7
TAR=http://download.netsurf-browser.org/netsurf/releases/source-full/netsurf-all-$VERSION.tar.gz
BUILD_DEPENDS=(curl expat libjpeg libpng openssl sdl zlib freetype liborbital libiconv)
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
    export LDFLAGS="-L$sysroot/lib -L${PWD}/inst-${TARGET}/lib -Wl,--allow-multiple-definition"
    make V=1 -j"$(nproc)"
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
    mkdir -pv "$dest/ui/apps"
    cp -v ../manifest "$dest/ui/apps/00_netsurf"
    skip=1
}
