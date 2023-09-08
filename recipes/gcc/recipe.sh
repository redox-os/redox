VERSION="13.2.0"
TAR="https://gitlab.redox-os.org/redox-os/gcc/-/archive/redox-${VERSION}/gcc-redox-${VERSION}.tar.gz"
#GIT=https://gitlab.redox-os.org/redox-os/gcc.git
#GIT_UPSTREAM=https://gcc.gnu.org/git/gcc.git
#BRANCH="redox-${VERSION}"
BUILD_DEPENDS=(relibc)
DEPENDS="gnu-binutils relibc"

function recipe_version {
    echo "${VERSION}"
    skip=1
}

function recipe_build {
    ./contrib/download_prerequisites
    cp config.sub gmp/config.sub
    cp config.sub isl/config.sub
    cp config.sub mpfr/config.sub
    cp -f config.sub mpc/config.sub

    sysroot="$(realpath ../sysroot)"
    mkdir -p "$sysroot/usr"
    ln -sf "$sysroot/include" "$sysroot/usr/include"
    ln -sf "$sysroot/lib" "$sysroot/usr/lib"
    export LDFLAGS="--static"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --target=${HOST} \
        --prefix=/ \
        --with-sysroot=/ \
        --with-build-sysroot="$sysroot" \
        --enable-static \
        --enable-shared \
        --disable-dlopen \
        --disable-nls \
        --enable-languages=c,c++ \
        --enable-threads=posix
    "$REDOX_MAKE" -j "$(nproc)" all-gcc all-target-libgcc all-target-libstdc++-v3
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install-gcc install-target-libgcc install-target-libstdc++-v3
    find "$dest"/{bin,libexec} -exec $STRIP {} ';' 2> /dev/null
    ln -s "gcc" "$1/bin/cc"
    skip=1
}
