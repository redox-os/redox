VERSION=2.6.6
TAR=https://files.freeciv.org/stable/freeciv-$VERSION.tar.bz2
BUILD_DEPENDS=(curl freetype2 libiconv liborbital libpng openssl1 nghttp2 sdl1 sdl-gfx sdl1-image sdl1-mixer sdl1-ttf zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    wget -O bootstrap/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib --static"
    export SDL_CONFIG="$sysroot/bin/sdl-config"
    ./configure \
        --build=${BUILD} \
        --host="$HOST" \
        --prefix='' \
        --disable-server \
        --enable-ipv6=no \
        --enable-client=sdl \
        --enable-fcmp=cli \
        --with-ft-prefix="$sysroot" \
        --with-sdl-prefix="$sysroot" \
        ac_cv_lib_SDL_image_IMG_Load=yes \
        ac_cv_lib_SDL_ttf_TTF_OpenFont=yes \
        ac_cv_lib_SDL_gfx_rotozoomSurface=yes \
        gui_sdl_cflags="${CFLAGS}" \
        gui_sdl_ldflags="${LDFLAGS}"
    "$REDOX_MAKE" -j"$($NPROC)" V=1
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
