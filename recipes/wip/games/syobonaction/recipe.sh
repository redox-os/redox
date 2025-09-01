VERSION=1.0-rc3
GIT=https://github.com/angelXwind/OpenSyobonAction
BUILD_DEPENDS=(sdl1 liborbital sdl1-mixer sdl1-image sdl-gfx sdl1-ttf freetype2 libjpeg libpng zlib libogg libvorbis)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export SDL_CONFIG="${PKG_CONFIG} sdl"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib --static"
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$1/bin"
    mkdir -pv "$1/share/syobonaction"
    cp -Rv ./SyobonAction "$1/bin/syobonaction"
    cp -Rv ./BGM "$1/share/syobonaction"
    cp -Rv ./res "$1/share/syobonaction"
    cp -Rv ./SE "$1/share/syobonaction"
    skip=1
}
