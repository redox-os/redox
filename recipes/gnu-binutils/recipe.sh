GIT=https://github.com/redox-os/binutils-gdb.git

HOST=x86_64-elf-redox
export AR="${HOST}-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-ld"
export NM="${HOST}-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export RANLIB="${HOST}-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    mkdir build
    cd build
    ../configure --host=${HOST} --target=${HOST} --prefix=/ --with-sysroot=/usr/x86_64-elf-redox --disable-gdb --disable-nls --disable-werror
    make
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    cd build
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    cd build
    make DESTDIR="$dest" install
    skip=1
}
