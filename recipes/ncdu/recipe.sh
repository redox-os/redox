VERSION=1.13
TAR=https://dev.yorhel.nl/download/ncdu-$VERSION.tar.gz
BUILD_DEPENDS=(ncurses)
DEPENDS=(terminfo)

function recipe_version {
    echo "$VERSION"
    skip=1
}
function recipe_build {
    sysroot="$PWD/../sysroot"
    export CPPFLAGS="-I$sysroot/include -I$sysroot/include/ncurses"
    export LDFLAGS="-L$sysroot/lib -static"
    ./configure \
        --build=${BUILD} \
        --host="$HOST" \
        --prefix=/
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}
function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}
function recipe_stage {
    dest="$(realpath "$1")"
    "$REDOX_MAKE" DESTDIR="$dest" install
    skip=1
}
