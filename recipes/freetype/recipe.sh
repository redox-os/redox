VERSION=2.9
TAR=https://download.savannah.gnu.org/releases/freetype/freetype-$VERSION.tar.gz
BUILD_DEPENDS=(zlib libpng)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export LDFLAGS="-L$sysroot/lib"
    export CPPFLAGS="-I$sysroot/include"

    ./configure --host=${HOST} --prefix='/'
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
    sed -i -e "s%//lib/libpng16.la%$dest/../sysroot/lib/libpng16.la%" "$dest/lib/libfreetype.la"
    skip=1
}
