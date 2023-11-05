GIT=https://gitlab.redox-os.org/redox-os/gawk.git
GIT_UPSTREAM=https://git.savannah.gnu.org/git/gawk.git
BRANCH=redox

function recipe_build {
    ./configure --build=${BUILD} --host=${HOST} --prefix=/ ac_cv_func_gethostbyname=no ac_cv_func_connect=no
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
