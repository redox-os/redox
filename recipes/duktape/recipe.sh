VERSION=2.2.0
TAR=http://duktape.org/duktape-$VERSION.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sed -i "s/= gcc/= $TARGET-gcc/g" Makefile.cmdline
    make -f Makefile.cmdline
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
    mkdir -pv "$1/bin"
    cp ./duk "$1/bin/duk"

    skip=1
}
