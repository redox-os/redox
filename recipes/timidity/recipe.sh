VERSION=2.15.0
GIT=https://gitlab.redox-os.org/redox-os/timidity.git
BRANCH=redox
DEPENDS="generaluser-gs"

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    export LDFLAGS="-static"
    autoreconf -f -i
    wget -O autoconf/config.sub "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix='' \
        --enable-vt100
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

    mkdir -pv "$1/etc/timidity"
    echo "soundfont /share/generaluser-gs/generaluser-gs.sf2" >> "$1/etc/timidity/timidity.cfg"

    mkdir -pv "$1/share/timidity"
    echo "soundfont /share/generaluser-gs/generaluser-gs.sf2" >> "$1/share/timidity/timidity.cfg"

    skip=1
}
