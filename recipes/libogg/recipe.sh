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
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"

    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=''
    "$REDOX_MAKE" -j"$($NPROC)"
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
    "$REDOX_MAKE" DESTDIR="$dest" install
    rm -f "$dest/lib/"*.la
    skip=1
}
