GIT=https://gitlab.redox-os.org/redox-os/mesa.git
GIT_UPSTREAM=git://anongit.freedesktop.org/mesa/mesa
BUILD_DEPENDS=(expat zlib)

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    NOCONFIGURE=1 ./autogen.sh
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export CPPFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export LDFLAGS="-L$sysroot/lib"
    EXPAT_LIBS="-lexpat" EXPAT_CFLAGS="." \
    ./configure --host="${HOST}" --prefix=/ \
        --disable-dri \
        --disable-dri3 \
        --disable-driglx-direct \
        --disable-egl \
        --disable-glx \
        --disable-gbm \
        --disable-llvm \
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
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    sed -i -e "s%//lib/libglapi.la%$dest/lib/libglapi.la%" "$dest/lib/"*.la
    skip=1
}
