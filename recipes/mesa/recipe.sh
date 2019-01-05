GIT=https://gitlab.redox-os.org/redox-os/mesa.git
GIT_UPSTREAM=git://anongit.freedesktop.org/mesa/mesa
GIT_BRANCH=redox
BUILD_DEPENDS=(expat llvm zlib)

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
    export CFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export CPPFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export LDFLAGS="-L$sysroot/lib"
    export LIBS="-Wl,--whole-archive -lpthread -Wl,--no-whole-archive"
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"
    NOCONFIGURE=1 ./autogen.sh
    ./configure \
        --host="${HOST}" \
        --prefix=/ \
        --disable-dri \
        --disable-dri3 \
        --disable-driglx-direct \
        --disable-egl \
        --disable-glx \
        --disable-gbm \
        --disable-llvm-shared-libs \
        --enable-llvm \
        --enable-gallium-osmesa \
        --with-gallium-drivers=swrast \
        --with-platforms=surfaceless
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
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
