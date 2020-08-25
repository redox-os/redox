VERSION=1.8
GIT=https://github.com/OpenTTD/OpenTTD.git
BRANCH=release/$VERSION
BUILD_DEPENDS=(freetype liborbital libpng sdl xz zlib)

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
    export LDFLAGS="-L$sysroot/lib --static"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix='' \
        --enable-static \
        --without-liblzo2 \
        --disable-network \
        --without-threads
    "$REDOX_MAKE" VERBOSE=1 -j"$($NPROC)"
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
    bundledir="$dest/bundle"

    "$REDOX_MAKE" VERBOSE=1 ROOT_DIR="$dest/../build/" BUNDLE_DIR="$bundledir" INSTALL_DIR="$dest" install
    rm -rf "$bundledir"
    skip=1
}
