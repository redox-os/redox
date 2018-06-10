GIT=https://gitlab.redox-os.org/redox-os/binutils-gdb.git

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --host=${HOST} --target=${HOST} --prefix=/ --with-sysroot=/usr/$HOST --disable-gdb --disable-nls --disable-werror
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
    find "$dest/bin" -exec $STRIP {} ';' 2> /dev/null
    skip=1
}
