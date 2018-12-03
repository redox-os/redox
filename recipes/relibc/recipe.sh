GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    make -j"$(nproc)"
    make -C tests -j"$(nproc)"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    mkdir -pv "$dest/share/relibc"
    cp -rv "tests" "$dest/share/relibc/tests"
    skip=1
}
