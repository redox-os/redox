VERSION=0.74
TAR=https://sourceforge.net/projects/dosbox/files/dosbox/$VERSION/dosbox-$VERSION.tar.gz/download
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
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include/SDL"
    export CPPFLAGS="-I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"
    ./autogen.sh
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix='' \
        --disable-opengl \
        --disable-sdltest \
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
