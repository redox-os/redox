GIT=https://github.com/abishekvashok/cmatrix
BUILD_DEPENDS=(ncurses)
DEPENDS=(terminfo)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib -static"
    export CPPFLAGS="-I$sysroot/include -I$sysroot/include/ncurses"
    autoreconf -i
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --without-fonts
    sed -i'' -e 's|#define USE_TIOCSTI 1|/* #undef USE_TIOCSTI */|g' config.h
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    skip=1
}
