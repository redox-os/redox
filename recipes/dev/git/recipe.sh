VERSION=2.13.1
TAR=https://www.kernel.org/pub/software/scm/git/git-$VERSION.tar.xz
BUILD_DEPENDS=(zlib curl openssl1 expat nghttp2)
DEPENDS="ca-certificates"

MAKEFLAGS=(
    NEEDS_SSL_WITH_CURL=1
    NEEDS_CRYPTO_WITH_SSL=1
    NO_IPV6=1
    NO_PREAD=1
    NO_MMAP=1
    NO_SETITIMER=1
    NO_UNIX_SOCKETS=1
    NEEDS_LIBICONV=
    NEEDS_LIBRT=
    BLK_SHA1=1
    V=1
)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-L$sysroot/lib -static"
    export CPPFLAGS="-I$sysroot/include"
    export CURL_CONFIG="$sysroot/bin/curl-config"
    ./configure \
        --build="${BUILD}" \
        --host="${HOST}" \
        --prefix=/ \
        ac_cv_fread_reads_directories=yes \
        ac_cv_snprintf_returns_bogus=yes \
        ac_cv_lib_curl_curl_global_init=yes
    "$REDOX_MAKE" "${MAKEFLAGS[@]}" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" "${MAKEFLAGS[@]}" install
    ${STRIP} $1/bin/* || true
    ${STRIP} $1/libexec/git-core/* || true
    rm -rf $1/share/man
    skip=1
}
