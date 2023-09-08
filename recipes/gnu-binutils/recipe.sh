GIT=https://gitlab.redox-os.org/redox-os/binutils-gdb.git
BRANCH=redox-2.41
GIT_UPSTREAM=git://sourceware.org/git/binutils-gdb.git
BUILD_DEPENDS=(expat libgmp)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    mkdir -p "$sysroot/usr"
    ln -sf "$sysroot/include" "$sysroot/usr/include"
    ln -sf "$sysroot/lib" "$sysroot/usr/lib"
    export CPPFLAGS="-I$sysroot/include -pie -fPIC -g"
    export LDFLAGS="-L$sysroot/lib --static -g"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --target=${HOST} \
        --prefix=/ \
        --with-sysroot=/ \
        --with-build-sysroot="$sysroot" \
        --enable-gdb \
        --with-expat \
        --with-multilib \
        --with-interwork \
        --enable-targets="${TARGET}" \
        --disable-nls \
        --disable-werror
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    find "$dest/bin" -exec $STRIP {} ';' 2> /dev/null
    skip=1
}
