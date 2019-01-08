VERSION=2.6.0
TAR=http://files.freeciv.org/stable/freeciv-$VERSION.tar.bz2
BUILD_DEPENDS=(curl freetype libiconv liborbital libpng openssl sdl sdl_gfx sdl_image sdl_mixer sdl_ttf zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O bootstrap/config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    ./configure \
        --host="$HOST" \
        --prefix='' \
        --disable-server \
        --enable-client=sdl \
        --enable-fcmp=cli \
        --with-sdl-prefix="$sysroot" \
        ac_cv_lib_SDL_image_IMG_Load=yes \
        ac_cv_lib_SDL_ttf_TTF_OpenFont=yes \
        ac_cv_lib_SDL_gfx_rotozoomSurface=yes
    make -j"$(nproc)" V=1
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
    make DESTDIR="$dest" install
    skip=1
}
