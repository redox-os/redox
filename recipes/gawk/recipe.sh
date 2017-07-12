GIT=https://github.com/redox-os/gawk
BRANCH=redox

HOST=x86_64-elf-redox

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --host=${HOST} --prefix=/ ac_cv_func_gethostbyname=no ac_cv_func_connect=no
    make
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    skip=1
}
