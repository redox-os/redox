GIT=https://gitlab.redox-os.org/redox-os/relibc.git

function recipe_build {
    make CARGO=xargo -j"$(nproc)"
    make CARGO=xargo -C tests -j"$(nproc)"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make CARGO=xargo DESTDIR="$dest" install
    mkdir -pv "$dest/share/relibc"
    cp -rv "tests" "$dest/share/relibc/tests"
    skip=1
}
