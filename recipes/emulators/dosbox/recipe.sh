VERSION=0.74-3
TAR=https://sourceforge.net/projects/dosbox/files/dosbox/$VERSION/dosbox-$VERSION.tar.gz/download
BUILD_DEPENDS=(sdl1 liborbital)

function recipe_version {
    echo "$VERSION"
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

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" DESTDIR="$dest" install

    mkdir -pv "$1/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/dosbox"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "${COOKBOOK_RECIPE}/icon.png" "$1/ui/icons/apps/dosbox.png"

    skip=1
}
