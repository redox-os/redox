VERSION=2.59.0
TAR=https://download.gnome.org/sources/glib/${VERSION%.*}/glib-$VERSION.tar.xz
BUILD_DEPENDS=(gettext libffi libiconv pcre zlib)

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
    glib_cv_stack_grows=no glib_cv_uscore=no ./autogen.sh \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static
    sed -i 's/#define HAVE_SYS_RESOURCE_H 1/#undef HAVE_SYS_RESOURCE_H/' config.h
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
    rm -f "$dest/lib/"*.la
    skip=1
}
