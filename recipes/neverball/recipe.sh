VERSION=1.6.0
TAR="https://neverball.org/neverball-${VERSION}.tar.gz"
BUILD_DEPENDS=(freetype libjpeg libogg liborbital libpng libvorbis llvm mesa sdl2 sdl2_ttf zlib)

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
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib -static"
    #TODO: Make sol using host compiler
    make -j"$(nproc)" ENABLE_FS=stdio ENABLE_NLS=0 neverball neverputt
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
