VERSION=1.3.6
TAR=http://downloads.xiph.org/releases/vorbis/libvorbis-$VERSION.tar.xz
TAR_SHA256=af00bb5a784e7c9e69f56823de4637c350643deedaf333d0fa86ecdba6fcb415
BUILD_DEPENDS=(libogg)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
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
