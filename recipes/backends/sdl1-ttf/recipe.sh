VERSION=2.0.11
TAR=https://www.libsdl.org/projects/SDL_ttf/release/SDL_ttf-$VERSION.tar.gz
BUILD_DEPENDS=(sdl1 liborbital freetype2 libpng zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/freetype2"
    export LDFLAGS="-L$sysroot/lib"
    ./autogen.sh
    ./configure --prefix=/ --build=${BUILD} --host=${HOST} --disable-shared
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
    rm -f "$dest/lib/"*.la
    skip=1
}
