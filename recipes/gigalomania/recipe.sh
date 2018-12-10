VERSION=0.27
GIT=https://github.com/fabiao/gigalomania
BRANCH=master
BUILD_DEPENDS=(sdl_mixer sdl_image sdl liborbital libpng libjpeg zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    export CPPHOST=${HOST}-g++
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include"
    make all -j"$(nproc)"
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
    bundledir="$dest/bundle"

    make VERBOSE=1 DESTDIR="$dest" install
    rm -rf "$bundledir"
    skip=1
}
