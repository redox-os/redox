VERSION=1.2.12
TAR=https://www.libsdl.org/projects/SDL_image/release/SDL_image-$VERSION.tar.gz
BUILD_DEPENDS=(sdl liborbital libjpeg libpng zlib)

export CFLAGS="-I$PWD/sysroot/include/"
export LDFLAGS="-L$PWD/sysroot/lib/"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./autogen.sh
    ./configure --prefix=/ --host=${HOST} --disable-shared --disable-sdltest --enable-png --enable-jpg
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
