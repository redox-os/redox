VERSION="7.62.0"
TAR=https://curl.haxx.se/download/curl-$VERSION.tar.gz
BUILD_DEPENDS=(openssl zlib)
DEPENDS="ca-certificates"

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
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    autoreconf -i
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-tftp \
        --disable-ftp \
        --disable-ntlm-wb \
        --disable-threaded-resolver \
        --with-zlib="$sysroot" \
        --with-ssl="$sysroot" \
        --with-ca-path=/ssl/certs
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
