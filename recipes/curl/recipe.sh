GIT=https://github.com/ids1024/curl.git
BRANCH=redox
BUILD_DEPENDS=(openssl)

HOST=x86_64-elf-redox

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
    mkdir "$1/bin"
    cp src/curl "$1/bin"
    skip=1
}
