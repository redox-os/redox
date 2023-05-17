VERSION=6.4
TAR=https://ftp.gnu.org/gnu/ncurses/ncurses-$VERSION.tar.gz
DEPENDS="terminfo"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix="" \
        --disable-db-install \
        --without-ada \
        --without-cxx-binding \
        --without-tests \
        cf_cv_func_mkstemp=yes
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
    rm -rf "$1"/bin
    rm -rf "$1"/share/{doc,info,man}
    skip=1
}
