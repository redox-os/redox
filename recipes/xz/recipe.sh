VERSION=5.2.3
TAR=https://codeload.github.com/xz-mirror/xz/tar.gz/v$VERSION

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    export CFLAGS="-static"
    
    # autogen.sh requires autopoint which is provided by the gettext homebrew
    # formula on macOS. Unfortunately, homebrew does not install it into PATH
    # because macOS provides the BSD gettext library. So we make sure to include
    # it in PATH, preceding the default BSD version.
    if [[ "$(uname)" == "Darwin" ]]; then
        export PATH="/usr/local/opt/gettext/bin:$PATH"
    fi

    ./autogen.sh

    chmod +w build-aux/config.sub
    wget -O build-aux/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --disable-lzmadec \
        --disable-lzmainfo \
        --disable-xz \
        --disable-xzdec \
        --enable-shared=no \
        --enable-static=yes \
        --enable-threads=no \
        --with-pic=no
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
    rm -rf "$dest/share"
    skip=1
}
