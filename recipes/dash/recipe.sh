GIT=https://gitlab.redox-os.org/redox-os/dash.git
BRANCH=redox

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./autogen.sh
    ./configure \
        --build=${BUILD} \
        --host=${HOST} \
        --prefix=/ \
        --enable-static \
        cross_compiling=yes

    # See https://stackoverflow.com/questions/4247068/sed-command-with-i-option-failing-on-mac-but-works-on-linux.
    sed -i'' -e 's|#define HAVE_GETRLIMIT 1|/* #undef HAVE_GETRLIMIT */|g' config.h
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
    ln -s "dash" "$dest/bin/sh"
    skip=1
}
