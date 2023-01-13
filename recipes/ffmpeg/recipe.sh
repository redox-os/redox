VERSION=4.0
GIT=https://github.com/FFmpeg/FFmpeg
BRANCH=release/$VERSION
BUILD_DEPENDS=(liborbital llvm mesa sdl2 zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export CPPFLAGS="-I${COOKBOOK_SYSROOT}/include"
    export LDFLAGS="-L${COOKBOOK_SYSROOT}/lib -static"
    ./configure \
        --enable-cross-compile \
        --target-os=redox \
        --arch=${ARCH} \
        --cross_prefix=${HOST}- \
        --prefix=/ \
        --disable-network \
        --enable-sdl2 \
        --enable-zlib \
        --enable-encoder=png \
        --enable-decoder=png
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
    skip=1
}
