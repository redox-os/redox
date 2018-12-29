VERSION=1.8
GIT=https://github.com/OpenTTD/OpenTTD.git
BRANCH=release/$VERSION
BUILD_DEPENDS=(sdl liborbital zlib xz)

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    ./configure --build=`gcc -dumpmachine` --host=${HOST} --prefix='' --enable-static --without-liblzo2 --disable-network --without-threads
    make VERBOSE=1 -j"$(nproc)"
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
    bundledir="$dest/bundle"

    make VERBOSE=1 ROOT_DIR="$dest/../build/" BUNDLE_DIR="$bundledir" INSTALL_DIR="$dest" install
    rm -rf "$bundledir"
    skip=1
}
