VERSION=2.0.4
TAR=https://www.libsdl.org/projects/SDL_mixer/release/SDL2_mixer-$VERSION.tar.gz
BUILD_DEPENDS=(sdl2 liborbital mesa mesa_glu zlib libogg libvorbis)

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
    export SDL_LIBS="-lSDL2 -lorbital $("${PKG_CONFIG}" --libs glu) -lvorbis -logg -lz -lm -lpthread -lstdc++"
    ./autogen.sh
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-shared \
        --disable-sdltest \
        --enable-music-ogg \
        --disable-music-cmd \
        --disable-music-mp3 \
        --disable-smpegtest \
        --disable-music-midi \
        --disable-music-mod
    "$REDOX_MAKE" -j"$($NPROC)"
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
    rm -f "$dest/lib/"*.la
    skip=1
}
