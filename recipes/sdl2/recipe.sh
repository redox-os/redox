VERSION=2.0.9
GIT=https://gitlab.redox-os.org/fabiao/sdl2-src.git
BUILD_DEPENDS=(liborbital mesa)

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
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --disable-pulseaudio \
        --disable-video-x11 \
        --disable-loadso \
        --disable-sdl-dlopen \
        --enable-threads \
        --enable-audio \
        --enable-dummyaudio \
        --enable-video-orbital \
        --enable-redoxaudio \
        --enable-cdrom
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
