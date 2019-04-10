VERSION="7.62.0"
#TAR=https://curl.haxx.se/download/curl-$VERSION.tar.gz
GIT=https://gitlab.redox-os.org/redox-os/curl.git
GIT_BRANCH=redox
BUILD_DEPENDS=(nghttp2 openssl zlib)
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
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    autoreconf -i
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-ftp \
        --disable-ipv6 \
        --disable-ntlm-wb \
        --disable-tftp \
        --disable-threaded-resolver \
        --with-ca-path=/ssl/certs \
        --with-nghttp2="$sysroot" \
        --with-ssl="$sysroot" \
        --with-zlib="$sysroot"
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
