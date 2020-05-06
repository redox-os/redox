VERSION=20181010-7067
TAR=https://dukeworld.com/eduke32/synthesis/$VERSION/eduke32_src_$VERSION.tar.xz
BUILD_DEPENDS=(sdl sdl_mixer liborbital libiconv libogg libvorbis)

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
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export LDFLAGS="-L$sysroot/lib -static"
    export SDLCONFIG="$sysroot/bin/sdl-config --prefix=$sysroot"

    PLATFORM=REDOX "$REDOX_MAKE" -j"$($NPROC)"
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
    mkdir -pv "$1/games"
    cp ./eduke32 "$1/games/eduke32"
    cp ./mapster32 "$1/games/mapster32"
    skip=1
}
