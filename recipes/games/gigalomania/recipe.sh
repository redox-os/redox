VERSION=1.0.1
GIT=https://gitlab.redox-os.org/redox-os/gigalomania.git
BRANCH=master
BUILD_DEPENDS=(sdl1-mixer sdl1-image sdl1 liborbital libogg libpng libjpeg libvorbis zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export CPPHOST=${HOST}-g++
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib --static"
    export CPPFLAGS="-I$sysroot/include"
    "$REDOX_MAKE" all -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    bundledir="$dest/bundle"

    "$REDOX_MAKE" VERBOSE=1 DESTDIR="$dest/usr" install
    rm -rf "$bundledir"

    mkdir -pv "$1/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/gigalomania"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "gigalomania64.png" "$1/ui/icons/apps/gigalomania.png"

    skip=1
}
