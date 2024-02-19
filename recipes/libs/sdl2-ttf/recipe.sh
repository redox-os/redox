VERSION=2.0.15
TAR=https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-$VERSION.tar.gz
BUILD_DEPENDS=(sdl2 liborbital llvm mesa freetype2 libpng zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    export SDL_LIBS="-lSDL2 -lorbital $("${PKG_CONFIG}" --libs osmesa) -lz -lm -lpthread -lstdc++"
    ./autogen.sh
    ./configure --prefix=/ --build=${BUILD} --host=${HOST} --enable-opengl --disable-shared --disable-sdltest
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
    rm -f "$dest/lib/"*.la
    skip=1
}
