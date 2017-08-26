GIT=https://github.com/FFmpeg/FFmpeg

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure \
        --enable-cross-compile \
        --target-os=redox \
        --arch=x86_64 \
        --cross_prefix=x86_64-unknown-redox- \
        --prefix=/ \
        --disable-doc \
        --disable-network \
        --disable-ffplay \
        --disable-ffprobe \
        --disable-ffserver
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
