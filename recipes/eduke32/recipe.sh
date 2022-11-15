VERSION=20181010-7067
TAR=https://dukeworld.com/eduke32/synthesis/$VERSION/eduke32_src_$VERSION.tar.xz
BUILD_DEPENDS=(sdl sdl_mixer liborbital libiconv libogg libvorbis)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"
    export SDLCONFIG="$sysroot/bin/sdl-config --prefix=$sysroot"

    PLATFORM=REDOX "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/games"
    cp -v ./eduke32 "$1/games/eduke32"
    cp -v ./mapster32 "$1/games/mapster32"

    mkdir -pv "$1/ui/apps"
    cp -v "${COOKBOOK_RECIPE}/manifest" "$1/ui/apps/eduke32"

    mkdir -pv "$1/ui/icons/apps"
    cp -v "${COOKBOOK_RECIPE}/icon.png" "$1/ui/icons/apps/eduke32.png"

    skip=1
}
