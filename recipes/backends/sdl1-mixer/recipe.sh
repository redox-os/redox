VERSION=1.2.12
TAR=https://www.libsdl.org/projects/SDL_mixer/release/SDL_mixer-$VERSION.tar.gz
BUILD_DEPENDS=(sdl1 liborbital libogg libvorbis)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    export LIBS="-lvorbis -logg"
    ./autogen.sh
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --enable-music-ogg \
        --enable-music-midi \
        --disable-shared \
        --disable-sdltest \
        --disable-music-cmd \
        --disable-music-mp3 \
        --disable-smpegtest \
        --disable-music-mod
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
