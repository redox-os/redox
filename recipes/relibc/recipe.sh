GIT=https://github.com/redox-os/relibc.git

function recipe_build {
    make
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/lib"
    mkdir -pv "$dest/include"
    cp -rv "include"/* "$dest/include"
    cp -rv "target/include"/* "$dest/include"
    cp -v "target/$TARGET/debug/libc.a" "$dest/lib"
    cp -v "target/$TARGET/debug/crt0.o" "$dest/lib"
    skip=1
}
