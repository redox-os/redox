VERSION=2.0.11
TAR=https://www.libsdl.org/projects/SDL_ttf/release/SDL_ttf-$VERSION.tar.gz
BUILD_DEPENDS=(sdl liborbital freetype libpng zlib)

export CFLAGS="-I$PWD/sysroot/include/ -I$PWD/sysroot/include/freetype2/"
export LDFLAGS="-L$PWD/sysroot/lib/"

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
    export SDL_CONFIG="$sysroot/bin/sdl-config"
    export FREETYPE_CONFIG="$sysroot/bin/freetype-config"

    ./autogen.sh
    ./configure --prefix=/ --host=${HOST} --disable-shared --disable-sdltest
    make -j"$(nproc)"
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
    sysroot="$(realpath ../sysroot)"
    make DESTDIR="$dest" install
    sed -i -e "s%//lib/libfreetype.la%$sysroot/lib/libfreetype.la%" "$dest/lib/"*.la
    sed -i -e "s%//lib/libpng16.la%$sysroot/lib/libpng16.la%" "$dest/lib/"*.la
    sed -i -e "s%//lib/libSDL.la%$sysroot/lib/libSDL.la%" "$dest/lib/"*.la
    skip=1
}
