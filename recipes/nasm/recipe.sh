VERSION=2.13.01
TAR=http://www.nasm.us/pub/nasm/releasebuilds/$VERSION/nasm-$VERSION.tar.gz

HOST=x86_64-elf-redox

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --host=${HOST} --prefix=""
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
    make INSTALLROOT="$dest" install
    rm -rf "$dest"/share
    find "$dest"/bin -exec ${HOST}-strip {} ';' 2> /dev/null
    skip=1
}
