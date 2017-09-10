GIT=https://github.com/FFmpeg/FFmpeg
BRANCH=release/3.3

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure \
        --enable-cross-compile \
        --target-os=redox \
        --arch=${ARCH} \
        --cross_prefix=${HOST}- \
        --prefix=/ \
        --disable-network
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
    rm -rf "$1"/{include,lib,share}
    skip=1
}
