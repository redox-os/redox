VERSION=1.8.4
TAR=https://sourceforge.net/projects/sdl-sopwith/files/sdl_sopwith/$VERSION/sopwith-$VERSION.tar.gz
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
    wget -O autotools/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib"

    ./configure --host=${HOST} --prefix='' --with-sdl-prefix="$sysroot"
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
