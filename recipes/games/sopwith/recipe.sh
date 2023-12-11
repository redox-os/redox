VERSION=1.8.4
TAR=https://sourceforge.net/projects/sdl-sopwith/files/sdl_sopwith/$VERSION/sopwith-$VERSION.tar.gz/download
BUILD_DEPENDS=(sdl1 liborbital libiconv)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    wget -O autotools/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"
    export LIBS="-lSDL -lorbital" # TODO: Uses sdl-config instead of pkg-config
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix='' \
        --with-sdl-prefix="$sysroot"
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
    skip=1
}
