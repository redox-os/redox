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
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib"
    export CFLAGS="-I$sysroot/include"
    ./configure --disable-shared --build=${BUILD} --host=${HOST} --prefix=""
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    rm -rf "$1"/share/{doc,info,man}
    skip=1
}
