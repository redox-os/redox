VERSION=1.13
TAR=https://dev.yorhel.nl/download/ncdu-$VERSION.tar.gz
BUILD_DEPENDS=(ncurses)
DEPENDS=(terminfo)

function recipe_version {
    echo "$VERSION"
    skip=1
}
function recipe_update {
    echo "skipping update"
    skip=1
}
function recipe_build {
    sysroot="$PWD/../sysroot"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include -I$sysroot/include/ncurses"
    ./configure \
        --build x86_64-pc-linux-gnu \
        --host "$HOST"
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
    dest="$(realpath "$1")"
    make DESTDIR="$dest" install
    cd "$dest/usr/local/bin/"
    find . -type f -exec install -D "{}" "$dest/usr/bin/{}" \;
    cd -
    cd "$dest/usr/local/share/"
    find . -type f -exec install -D "{}" "$dest/share/{}" \;
    cd -
    rm -r "$dest/usr/local/"
    skip=1
}
