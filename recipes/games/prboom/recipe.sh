VERSION=2.5.0
TAR=https://downloads.sourceforge.net/project/prboom/prboom%20stable/$VERSION/prboom-$VERSION.tar.gz
BUILD_DEPENDS=(sdl1 liborbital sdl1-mixer libogg libvorbis)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export CFLAGS="-static"
    export MIXER_LIBS="-lSDL_mixer -lvorbisfile -lvorbis -logg"
    sysroot="$(realpath ../sysroot)"
    autoreconf -if
    wget -O autotools/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-cpu-opt \
        --disable-i386-asm \
        --disable-gl \
        --disable-sdltest \
        --without-net \
        --with-sdl-prefix="$sysroot" \
        ac_cv_lib_SDL_mixer_Mix_OpenAudio=yes
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest/usr" install
    skip=1
}
