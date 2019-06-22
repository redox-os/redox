GIT=https://gitlab.redox-os.org/redox-os/binutils-gdb.git
GIT_UPSTREAM=git://sourceware.org/git/binutils-gdb.git
BUILD_DEPENDS=(relibc)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
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
        --disable-gdb \
        --disable-nls \
        --disable-werror
    make -j"$(nproc)"
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
    find "$dest/bin" -exec $STRIP {} ';' 2> /dev/null
    skip=1
}
