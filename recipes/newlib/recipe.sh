GIT=https://github.com/redox-os/newlib.git
BRANCH=redox

CC=

HOST=x86_64-elf-redox

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    return 1
}

function recipe_update {
    echo "skipping update"
    return 1
}

function recipe_build {
    pushd newlib/libc/sys
        aclocal-1.11 -I ../..
        autoconf
        automake-1.11 --cygnus Makefile
    popd

    pushd newlib/libc/sys/redox
        aclocal-1.11 -I ../../..
        autoconf
        automake-1.11 --cygnus Makefile
    popd

    mkdir build
    cd build

    ../configure --target="${HOST}" --prefix=/
    make

    return 1
}

function recipe_test {
    echo "skipping test"
    return 1
}

function recipe_clean {
    cd build
    make clean
    return 1
}

function recipe_stage {
    dest="$(realpath $1)"
    cd build
    make DESTDIR="$dest" install
    cd "$dest"
    mv x86_64-elf-redox/* ./
    rmdir x86_64-elf-redox
    return 1
}
