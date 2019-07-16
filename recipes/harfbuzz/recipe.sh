VERSION=2.3.0
TAR=https://www.freedesktop.org/software/harfbuzz/release/harfbuzz-$VERSION.tar.bz2
BUILD_DEPENDS=(freetype gettext glib libiconv libpng pcre zlib)

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
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib --static"
    #wget -O build-aux/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    FREETYPE_CFLAGS="$("${PKG_CONFIG}" --cflags freetype2)"
    FREETYPE_LIBS="$("${PKG_CONFIG}" --libs freetype2)"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static \
        --with-glib=yes \
        --with-freetype=yes \
        --with-icu=no
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
    rm -f "$dest/lib/"*.la
    skip=1
}
