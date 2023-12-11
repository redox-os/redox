VERSION=0.7
GIT=https://github.com/mgba-emu/mgba.git
BRANCH=$VERSION
BUILD_DEPENDS=(sdl1 liborbital libiconv pixman)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"

    mkdir -p build
    cd build
    cmake \
        -DCMAKE_INSTALL_PREFIX:PATH=/ \
        -DBUILD_STATIC=ON \
        -DBUILD_SHARED=OFF \
        -DBUILD_QT=OFF \
        -DUSE_SQLITE3=OFF \
        -DUSE_DEBUGGERS=OFF \
        -DBUILD_SDL=ON \
        -DSDL_VERSION="1.2" \
        -DSDL_LIBRARY="-lSDL -lorbital" \
        ..
    VERBOSE=1 "$REDOX_MAKE" all -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp "../build/build/sdl/mgba" "$dest/bin/mgba"
    skip=1
}
