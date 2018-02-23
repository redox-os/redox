VERSION=3.6.2
TAR=https://www.python.org/ftp/python/$VERSION/Python-$VERSION.tar.xz

export CONFIG_SITE=config.site

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    cp ../config.site ./
    ./configure --host=${HOST} --build=${ARCH} --prefix=/
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
    make prefix="$dest" install
    $STRIP "$dest/bin/python3.6"
    rm -rf "$dest"/{share,lib/*.a,include}
    skip=1
}
