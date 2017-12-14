GIT=https://github.com/redox-os/newlibtest.git
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
    export CC="${HOST}-gcc"
    export LD="${HOST}-ld"
    export CFLAGS="-nostdinc -nostdlib -static $sysroot/lib/crt0.o"
    export LIBS="-I $sysroot/include -L $sysroot/lib -lc -lm"

    make all
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
