VERSION=3.1.0
TAR=https://download.qemu.org/qemu-$VERSION.tar.xz
BUILD_DEPENDS=(curl glib libiconv libpng pcre pixman sdl1 zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    ./configure \
        --build=${BUILD} \
        --host="${HOST}" \
        --prefix=/
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
