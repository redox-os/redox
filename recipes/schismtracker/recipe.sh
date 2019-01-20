VERSION=20181223
TAR=https://github.com/schismtracker/schismtracker/archive/$VERSION.tar.gz
TAR_SHA256=fc32930c611fdb78face87dbe8a3c62e70088fd8d4ad803140e0b9a0b2e72ad7
BUILD_DEPENDS=(sdl liborbital libiconv)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib"
    export SDL_CONFIG="$sysroot/bin/sdl-config"
    autoreconf -i
    ./configure --build=${BUILD} --host=${HOST} --prefix=''
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
    skip=1
}
