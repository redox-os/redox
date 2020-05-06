GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    "$REDOX_MAKE" CARGO=xargo NATIVE_RELIBC=1 -C tests -j"$($NPROC)"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/share/relibc"
    cp -rv "tests" "$dest/share/relibc/tests"
    skip=1
}
