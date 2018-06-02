VERSION=1.0.16
TAR=https://github.com/jedisct1/libsodium/archive/${VERSION}.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    # Disclaimer: No idea what I'm doing
    ./autogen.sh
    ./configure --host=${HOST} --prefix='/'
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
    skip=1
}
