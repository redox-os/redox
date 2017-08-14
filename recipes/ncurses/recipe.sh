VERSION=6.0
TAR=http://ftp.gnu.org/gnu/ncurses/ncurses-$VERSION.tar.gz

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
    make DESTDIR="$dest" install
    rm -rf "$1"/bin
    rm -rf "$1"/share/{doc,info,man}
    skip=1
}
