VERSION=7.6p1
TAR=http://ftp.openbsd.org/pub/OpenBSD/OpenSSH/portable/openssh-$VERSION.tar.gz
BUILD_DEPENDS=(zlib openssl)

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

function newlib_build {
    rm -rf ../newlib
    sysroot="${PWD}/../sysroot"
    cd ..
    git clone --recursive https://github.com/sajattack/newlib -b ssh-deps
    cd newlib
    pushd newlib/libc/sys
        aclocal-1.11 -I ../..
        autoconf
        automake-1.11 --cygnus Makefile
    popd

    pushd newlib/libc/sys/redox
        aclocal-1.11 -I ../../..
        autoconf
        automake-1.11 --cygnus Makefile
    popd

    CC= ./configure --target="${HOST}" --prefix=/
    make all
    make DESTDIR="$sysroot" install
    cd ..
    cp -r $sysroot/x86_64-unknown-redox/* $sysroot
    rm -rf $sysroot/x86_64-unknown-redox
    rm -rf newlib
    cd build
}

function recipe_build {
    newlib_build 
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
