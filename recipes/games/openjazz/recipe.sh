#TODO: support cmake version 20231028
VERSION=20190106
TAR="https://github.com/AlisterT/openjazz/releases/download/${VERSION}/openjazz-${VERSION}.tar.xz"
TAR_SHA256="91341adcc4908db12aad6b82d2fb0125429a26585f65d7eb32d403656313eaab"
BUILD_DEPENDS=(sdl1 liborbital zlib)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -UUSE_SOCKETS -UUSE_SDL_NET"
    export CPPFLAGS="$CFLAGS"
    export LDFLAGS="-L$sysroot/lib -static"
    touch INSTALL NEWS README AUTHORS ChangeLog COPYING
    autoreconf -fvi
    autoconf
    wget -O build-aux/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure --build=${BUILD} --host=${HOST} --prefix=''
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
