VERSION=5.3.1
TAR=http://www.lua.org/ftp/lua-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    make generic CC="${HOST}-gcc -std=gnu99"
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
    cp src/lua src/luac "$1/bin"
    skip=1
}
