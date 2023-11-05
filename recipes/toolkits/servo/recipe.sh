GIT=https://gitlab.redox-os.org/redox-os/servo.git
GIT_UPSTREAM=https://github.com/servo/servo.git
BRANCH=redox
BUILD_DEPENDS=(freetype gettext glib gstreamer harfbuzz libffi libiconv libpng openssl pcre zlib)
PREPARE_COPY=0

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_build {
    source="$(realpath ../source)"
    unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP
    "$source/mach" build --target "${TARGET}" --release --with-frame-pointer # --jobs "$(nproc)"
    skip=1
}

function recipe_clean {
    echo "skipping clean"
    skip=1
}

function recipe_stage {
    echo "skipping stage"
    skip=1
}
