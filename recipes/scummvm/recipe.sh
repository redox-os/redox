VERSION=2.0.0
TAR=https://www.scummvm.org/frs/scummvm/$VERSION/scummvm-$VERSION.tar.xz
TAR_SHA256=9784418d555ba75822d229514a05cf226b8ce1a751eec425432e6b7e128fca60
BUILD_DEPENDS=(sdl liborbital freetype zlib libpng)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    wget -O config.sub http://git.savannah.gnu.org/cgit/config.git/plain/config.sub
    sysroot="$(realpath ../sysroot)"

    ./configure \
        --host=${HOST} \
        --prefix='' \
        --with-sdl-prefix="$sysroot" \
        --with-freetype2-prefix="$sysroot" \
        --with-png-prefix="$sysroot" \
        --with-zlib-prefix="$sysroot" \
        --disable-timidity \
        --disable-mt32emu
    make -j"$(nproc)"
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
