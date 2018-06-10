GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    make
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    skip=1
}
