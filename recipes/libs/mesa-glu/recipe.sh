VERSION=9.0.1
TAR=https://archive.mesa3d.org/glu/glu-$VERSION.tar.xz
BUILD_DEPENDS=(mesa)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include"
    export CPPFLAGS="-I$sysroot/include"
    export LDFLAGS="-L$sysroot/lib"
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure --build=${BUILD} --host="${HOST}" --prefix=/ --enable-osmesa
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
