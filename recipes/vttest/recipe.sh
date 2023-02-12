VERSION=20140305
TAR="https://invisible-island.net/archives/vttest/vttest-${VERSION}.tgz"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export LDFLAGS="-static"
    wget -O config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=''
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
    skip=1
}
