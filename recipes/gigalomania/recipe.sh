VERSION=0.27
GIT=https://gitlab.redox-os.org/redox-os/gigalomania.git
BRANCH=master
BUILD_DEPENDS=(sdl_mixer sdl_image sdl liborbital libogg libpng libjpeg libvorbis zlib)

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
    "$REDOX_MAKE" all -j"$($NPROC)"
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

    "$REDOX_MAKE" VERBOSE=1 DESTDIR="$dest" install
    rm -rf "$bundledir"
    skip=1
}
