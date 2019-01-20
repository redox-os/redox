VERSION=2.14.02
TAR=http://www.nasm.us/pub/nasm/releasebuilds/$VERSION/nasm-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --build=${BUILD} --host=${HOST} --prefix=""
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
    find "$dest"/bin -exec ${HOST}-strip {} ';' 2> /dev/null
    skip=1
}
