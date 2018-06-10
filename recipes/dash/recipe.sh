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
    ./configure --host=${HOST} --prefix=/
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
    ln -s "dash" "$1/bin/sh"
    skip=1
}
