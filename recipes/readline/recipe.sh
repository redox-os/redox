VERSION=7.0
TAR=http://ftp.gnu.org/gnu/readline/readline-$VERSION.tar.gz
BUILD_DEPENDS=(ncurses)

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
    export LDFLAGS="-L$sysroot/lib"
    export CFLAGS="-I$sysroot/include"
    ./configure --disable-shared --host=${HOST} --prefix=""
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
    rm -rf "$1"/share/{doc,info,man}
    skip=1
}
