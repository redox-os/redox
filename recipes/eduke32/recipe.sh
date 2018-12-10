VERSION=20181010-7067
TAR=https://dukeworld.com/eduke32/synthesis/$VERSION/eduke32_src_$VERSION.tar.xz
BUILD_DEPENDS=(sdl sdl_mixer liborbital libiconv)

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
    export LDFLAGS="-L$sysroot/lib"
    export CFLAGS="-I$sysroot/include -I$sysroot/include/SDL"
    export SDLCONFIG="$sysroot/bin/sdl-config"

    PLATFORM=REDOX make -j"$(nproc)"
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
    mkdir -pv "$1/games"
    cp ./eduke32 "$1/games/eduke32"
    cp ./mapster32 "$1/games/mapster32"
    skip=1
}
