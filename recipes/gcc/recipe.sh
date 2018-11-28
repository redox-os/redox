GIT=https://gitlab.redox-os.org/redox-os/gcc.git
BRANCH=redox
BUILD_DEPENDS=(relibc)
DEPENDS="gnu-binutils relibc"

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./contrib/download_prerequisites
    cp config.sub gmp/config.sub
    cp config.sub isl/config.sub
    cp config.sub mpfr/config.sub
    cp -f config.sub mpc/config.sub

    sysroot="$(realpath "${PWD}/../sysroot")"
    mkdir -p "$sysroot/usr"
    ln -sf "$sysroot/include" "$sysroot/usr/include"
    ln -sf "$sysroot/lib" "$sysroot/usr/lib"
    ./configure --host=${HOST} --target=${HOST} --prefix=/ --with-sysroot=/ --with-build-sysroot="$sysroot" --enable-static --disable-shared --disable-dlopen --disable-nls --enable-languages=c,c++
    make -j "$(nproc)" all-gcc all-target-libgcc all-target-libstdc++-v3
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
    make DESTDIR="$dest" install-gcc install-target-libgcc install-target-libstdc++-v3
    find "$dest"/{bin,libexec} -exec $STRIP {} ';' 2> /dev/null
    ln -s "gcc" "$1/bin/cc"
    skip=1
}
