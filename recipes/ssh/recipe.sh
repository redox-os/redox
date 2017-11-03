VERSION=7.6p1
TAR=http://ftp.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-$VERSION.tar.gz
BUILD_DEPENDS=(zlib openssl newlib)

export AR="${HOST}-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-gcc"
export NM="${HOST}-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export RANLIB="${HOST}-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include"
    ./configure --host=${HOST} --prefix=/
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
