GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    make CARGO=xargo -j"$(nproc)"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make CARGO=xargo DESTDIR="$dest" install
    skip=1
}
