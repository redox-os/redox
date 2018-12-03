GIT=https://gitlab.redox-os.org/redox-os/newlibtest.git
BUILD_DEPENDS=(newlib)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="${PWD}/../sysroot"
    export CFLAGS="-static -nostdinc -I $sysroot/include -I /usr/lib/gcc/x86_64-unknown-redox/7.0.1/include/ -nostdlib -L $sysroot/lib"
    export CRT="$sysroot/lib/crt0.o"
    export CLIBS="-lc -lm"
    make all -j"$(nproc)"
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
    make DESTDIR="$dest" prefix=/ install
    skip=1
}
