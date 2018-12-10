VERSION=2.0.9
TAR=https://www.libsdl.org/release/SDL2-$VERSION.tar.gz
BUILD_DEPENDS=(liborbital)

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
    ./autogen.sh
    ./configure \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --disable-pulseaudio \
        --disable-video-x11 \
        --disable-loadso \
        --disable-sdl-dlopen \
        --disable-threads \
        --enable-audio \
        --enable-dummyaudio \
        --enable-video-orbital \
        --enable-cdrom
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
    make DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
