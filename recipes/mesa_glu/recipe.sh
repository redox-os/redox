TAR=ftp://ftp.freedesktop.org/pub/mesa/glu/glu-9.0.0.tar.bz2
BUILD_DEPENDS=(mesa)

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-I$sysroot/include"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    ./configure --host="${HOST}" --prefix=/ --enable-osmesa
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
