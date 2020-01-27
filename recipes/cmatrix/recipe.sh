GIT=https://github.com/abishekvashok/cmatrix
BUILD_DEPENDS=(ncurses)
DEPENDS=(terminfo)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
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
