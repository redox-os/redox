VERSION=4.2.1
TAR=https://ftp.gnu.org/gnu/make/make-$VERSION.tar.gz

HOST=x86_64-elf-redox

export AR="${HOST}-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-ld"
export NM="${HOST}-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export RANLIB="${HOST}-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

function recipe_version {
    echo "$VERSION"
    return 1
}

function recipe_update {
    echo "skipping update"
    return 1
}

function recipe_build {
    ./configure --host=${HOST} --prefix=/ CFLAGS=-DPOSIX --without-guile
    make
    return 1
}

function recipe_test {
    echo "skipping test"
    return 1
}

function recipe_clean {
    make clean
    return 1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    return 1
}
