VERSION=4.0
GIT=https://github.com/FFmpeg/FFmpeg
BRANCH=release/$VERSION
BUILD_DEPENDS=(zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="$PWD/../sysroot"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib -static"
    ./configure \
        --enable-cross-compile \
        --target-os=redox \
        --arch=${ARCH} \
        --cross_prefix=${HOST}- \
        --prefix=/ \
        --disable-network \
        --enable-zlib \
        --enable-encoder=png \
        --enable-decoder=png
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
    skip=1
}
