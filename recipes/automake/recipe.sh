VERSION=1.16.5
TAR=https://ftp.gnu.org/gnu/automake/automake-$VERSION.tar.xz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    wget -O lib/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    sed -i 's|.*/doc/help2man.*|\&\& true|' Makefile.in
    sed -i 's|install-info-am install-man|install-info-am|' Makefile.in

    ./configure --build=${BUILD} --host=${HOST} --prefix=''
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
