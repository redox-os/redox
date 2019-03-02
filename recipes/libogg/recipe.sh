VERSION=1.3.3
TAR=http://downloads.xiph.org/releases/ogg/libogg-$VERSION.tar.xz
TAR_SHA256=4f3fc6178a533d392064f14776b23c397ed4b9f48f5de297aba73b643f955c08

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

    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=''
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
    rm -f "$dest/lib/"*.la
    skip=1
}
