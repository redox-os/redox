VERSION=1.37.0
TAR=https://github.com/nghttp2/nghttp2/releases/download/v${VERSION}/nghttp2-${VERSION}.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure \
        --build="${BUILD}" \
        --host="${HOST}" \
        --prefix=/ \
        --enable-lib-only
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
    make install DESTDIR="$dest"
    rm -f "$dest/lib/"*.la
    skip=1
}
