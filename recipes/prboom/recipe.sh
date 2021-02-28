VERSION=2.5.0
TAR=https://downloads.sourceforge.net/project/prboom/prboom%20stable/$VERSION/prboom-$VERSION.tar.gz
BUILD_DEPENDS=(sdl liborbital)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    export CFLAGS="-static"
    sysroot="$(realpath ../sysroot)"
    autoreconf -if
    wget -O autotools/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --prefix=/ \
        --build=${BUILD} \
        --host=${HOST} \
        --disable-sdltest \
        --disable-cpu-opt \
        --disable-gl \
        --without-net \
        --with-sdl-prefix="$sysroot"
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_test {
    echo "skipping test"
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
