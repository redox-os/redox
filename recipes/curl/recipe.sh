VERSION="7.62.0"
#TAR=https://curl.haxx.se/download/curl-$VERSION.tar.gz
GIT=https://gitlab.redox-os.org/redox-os/curl.git
BRANCH=redox
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
    export LDFLAGS="-L$sysroot/lib --static"
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    autoreconf -i
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-ftp \
        --disable-ipv6 \
        --disable-ntlm-wb \
        --disable-shared \
        --disable-tftp \
        --disable-threaded-resolver \
        --enable-static \
        --with-ca-path=/ssl/certs \
        --with-nghttp2="$sysroot" \
        --with-ssl="$sysroot" \
        --with-zlib="$sysroot"
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
