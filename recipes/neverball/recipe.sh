VERSION=1.6.0
TAR="https://neverball.org/neverball-${VERSION}.tar.gz"
BUILD_DEPENDS=(freetype libjpeg libogg liborbital libpng libvorbis mesa sdl2 sdl2_ttf zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    env -i PATH=/usr/bin:/bin PKG_CONFIG=pkg-config \
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio mapc sols
    sysroot="$(realpath ../sysroot)"
    export CPPFLAGS="-I$sysroot/include"
	export LDFLAGS="-L$sysroot/lib -static"
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio ENABLE_NLS=0 clean-src
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio ENABLE_NLS=0 neverball neverputt
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
    mkdir -p "${dest}/games/neverball"
    for bin in neverball neverputt
    do
        "${STRIP}" -v "$bin" -o "${dest}/games/neverball/$bin"
    done
    cp -rv data "${dest}/games/neverball"
    skip=1
}
