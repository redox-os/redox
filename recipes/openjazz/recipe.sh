VERSION=0.1
GIT=https://github.com/AlisterT/openjazz
BUILD_DEPENDS=(sdl liborbital zlib)

export CFLAGS="-I$PWD/sysroot/include/ -I$PWD/sysroot/include/SDL/ -UUSE_SOCKETS -UUSE_SDL_NET"
export CPPFLAGS="$CFLAGS"
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
    touch INSTALL NEWS README AUTHORS ChangeLog COPYING
    autoreconf -fvi
    autoconf
    wget -O build-aux/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    ./configure --host=${HOST} --prefix=''
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
