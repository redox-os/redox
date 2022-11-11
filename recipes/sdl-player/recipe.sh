GIT=https://gitlab.redox-os.org/redox-os/sdl-player.git
BUILD_DEPENDS=(ffmpeg liborbital sdl zlib)

function recipe_version {
    echo "1.0.0"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib -static"
    "$REDOX_MAKE" -j"$($NPROC)"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "player" "$dest/bin/sdl-player"
    skip=1
}
