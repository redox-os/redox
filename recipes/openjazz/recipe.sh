VERSION=0.1
GIT=https://github.com/AlisterT/openjazz
BUILD_DEPENDS=(sdl liborbital zlib)

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
    export CFLAGS="-I$sysroot/include -UUSE_SOCKETS -UUSE_SDL_NET"
    export CPPFLAGS="$CFLAGS"
    export LDFLAGS="-L$sysroot/lib -static"
    touch INSTALL NEWS README AUTHORS ChangeLog COPYING
    autoreconf -fvi
    autoconf
    wget -O builds/autotools/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure --build=${BUILD} --host=${HOST} --prefix=''
    "$REDOX_MAKE" -j"$($NPROC)" V=1
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
    skip=1
}
