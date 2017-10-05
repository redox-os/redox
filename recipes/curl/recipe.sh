TAR=https://curl.haxx.se/download/curl-7.55.1.tar.gz
BRANCH=redox
BUILD_DEPENDS=(openssl)
DEPENDS="ca-certificates"

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    autoreconf -i
    ./configure --prefix=/ --host=${HOST} --disable-tftp --disable-ftp --disable-ntlm-wb --disable-threaded-resolver --with-ssl="$PWD/../sysroot" --with-ca-path=/ssl/certs
    make
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
    rm -rf "$1"/{share,lib/pkgconfig}
    skip=1
}
