GIT=https://github.com/redox-os/gcc.git
BRANCH=redox

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
    ./contrib/download_prerequisites
    cp config.sub gmp/config.sub
    cp config.sub isl/config.sub
    cp config.sub mpfr/config.sub
    cp -f config.sub mpc/config.sub
    pushd libstdc++-v3
    autoconf2.64
    popd

    ./configure --host=${HOST} --target=${HOST} --prefix=/ --enable-static --disable-shared --disable-dlopen --disable-nls --enable-languages=c --without-headers
    make all-gcc all-target-libgcc
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
    make DESTDIR="$dest" install-gcc install-target-libgcc
    find "$dest"/{bin,libexec} -exec x86_64-elf-redox-strip {} ';' 2> /dev/null
    skip=1
}
