VERSION=2.69
TAR=http://ftp.gnu.org/gnu/autoconf/autoconf-$VERSION.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O build-aux/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    ./configure --host=${HOST} --prefix=''
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
