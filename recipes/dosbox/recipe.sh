VERSION=0.74
TAR=https://sourceforge.net/projects/dosbox/files/dosbox/$VERSION/dosbox-$VERSION.tar.gz/download
BUILD_DEPENDS=(sdl liborbital)

export CFLAGS="-I$PWD/sysroot/include/SDL/"
export CPPFLAGS="-I$PWD/sysroot/include/SDL/"
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
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    ./configure --host=${HOST} --prefix='' --disable-opengl --disable-sdltest --with-sdl-prefix="$PWD/../sysroot"
    make
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
