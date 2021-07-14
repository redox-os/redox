GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    "$REDOX_MAKE" CARGO="env -u CARGO cargo" -j"$($NPROC)"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    "$REDOX_MAKE" CARGO="env -u CARGO cargo" DESTDIR="$dest" install
    skip=1
}
