VERSION=4.2.1
TAR=https://ftp.gnu.org/gnu/make/make-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --host=${HOST} --prefix=/ CFLAGS="-DPOSIX -DNO_ARCHIVES -DNO_OUTPUT_SYNC" --without-guile
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
    skip=1
}
