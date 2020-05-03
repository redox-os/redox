GIT=https://gitlab.redox-os.org/redox-os/mesa.git
GIT_UPSTREAM=git://anongit.freedesktop.org/mesa/mesa
BRANCH=redox
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
    export LDFLAGS="-L$sysroot/lib --static"
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"
    NOCONFIGURE=1 ./autogen.sh
    ./configure \
        --build=${BUILD} \
        --host="${HOST}" \
        --prefix=/ \
        --disable-dri \
        --disable-dri3 \
        --disable-driglx-direct \
        --disable-egl \
        --disable-glx \
        --disable-gbm \
        --disable-llvm-shared-libs \
        --disable-shared \
        --enable-llvm \
        --enable-gallium-osmesa \
        --enable-static \
        --with-gallium-drivers=swrast \
        --with-platforms=surfaceless
    $REDOX_MAKE V=1 -j"$($NPROC)"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    $REDOX_MAKE clean
    skip=1
}

function recipe_stage {
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"
    dest="$(realpath $1)"
    $REDOX_MAKE DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
