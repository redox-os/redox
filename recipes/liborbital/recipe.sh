GIT=https://github.com/xtibor/liborbital

function recipe_stage {
    dest="$(realpath $1)"
    make HOST="$HOST" DESTDIR="$dest" install
    skip=1
}
