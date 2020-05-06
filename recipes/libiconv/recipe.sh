VERSION=1.15
TAR=https://ftp.gnu.org/pub/gnu/libiconv/libiconv-$VERSION.tar.gz

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    export LDFLAGS="--static"
    ./configure \
        --build="${BUILD}" \
        --host="${HOST}" \
        --prefix='/' \
        --disable-shared \
        --enable-static \
        ac_cv_have_decl_program_invocation_name=no
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
