VERSION=1.6.0
TAR="https://neverball.org/neverball-${VERSION}.tar.gz"
BUILD_DEPENDS=(freetype2 libjpeg libogg liborbital libpng libvorbis llvm mesa sdl2 sdl2-ttf zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    set -x
    env -i \
        LDFLAGS="-static" \
        PATH="/usr/bin:/bin" \
        PKG_CONFIG="pkg-config" \
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio mapc sols
    sysroot="$(realpath ../sysroot)"
    export CPPFLAGS="-I$sysroot/include"
	export LDFLAGS="-L$sysroot/lib -static -z noexecstack"
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio ENABLE_NLS=0 clean-src
    "$REDOX_MAKE" -j"$($NPROC)" ENABLE_FS=stdio ENABLE_NLS=0 neverball neverputt
    set +x
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"

    # Create install directories
    mkdir -pv "${dest}/games/neverball" "${dest}/ui/apps" "${dest}/ui/icons/apps"

    # Copy assets
    cp -rv data "${dest}/games/neverball"

    # For each game
    for bin in neverball neverputt
    do
        # Install binary
        "${STRIP}" -v "${bin}" -o "${dest}/games/neverball/${bin}"

        # Install manifest
        cp -v "${COOKBOOK_RECIPE}/manifest-${bin}" "${dest}/ui/apps/${bin}"

        # Install icon
        cp -v "dist/${bin}_64.png" "${dest}/ui/icons/apps/${bin}.png"
    done

    skip=1
}
