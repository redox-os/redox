GIT=https://github.com/ids1024/curl.git
BRANCH=redox
BUILD_DEPENDS=(openssl)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --prefix=/ --host=${HOST} --disable-tftp --disable-ftp --disable-ntlm-wb --with-ssl="$PWD/../sysroot" --with-ca-path=/ssl/certs
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
