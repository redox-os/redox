VERSION=8.42
TAR=https://ftp.pcre.org/pub/pcre/pcre-$VERSION.tar.bz2
BUILD_DEPENDS=()

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
    export LDFLAGS="-L$sysroot/lib"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-shared \
        --enable-static
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
