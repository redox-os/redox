VERSION=20181223
TAR=https://github.com/schismtracker/schismtracker/archive/$VERSION.tar.gz
TAR_SHA256=fc32930c611fdb78face87dbe8a3c62e70088fd8d4ad803140e0b9a0b2e72ad7
BUILD_DEPENDS=(sdl1 liborbital libiconv)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"
    export SDL_CONFIG="$sysroot/bin/sdl-config"
    autoreconf -i
    ./configure --build=${BUILD} --host=${HOST} --prefix=''
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

    mkdir -pv "$1/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/schismtracker"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "icons/schism-icon-64.png" "$1/ui/icons/apps/schismtracker.png"

    skip=1
}
