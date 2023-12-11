VERSION=2.0.0
TAR=https://downloads.scummvm.org/frs/scummvm/$VERSION/scummvm-$VERSION.tar.xz
TAR_SHA256=9784418d555ba75822d229514a05cf226b8ce1a751eec425432e6b7e128fca60
BUILD_DEPENDS=(sdl1 liborbital freetype2 zlib libpng)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    sysroot="$(realpath ../sysroot)"
    export LDFLAGS="-static"
    ./configure \
        --host=${HOST} \
        --prefix='' \
        --with-sdl-prefix="$sysroot" \
        --with-freetype2-prefix="$sysroot" \
        --with-png-prefix="$sysroot" \
        --with-zlib-prefix="$sysroot" \
        --disable-timidity \
        --disable-mt32emu
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
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/scummvm"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "${COOKBOOK_RECIPE}/icon.png" "$1/ui/icons/apps/scummvm.png"

    skip=1
}
