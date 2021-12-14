GIT=https://gitlab.redox-os.org/redox-os/rust.git
BRANCH=redox-2021-06-15
BUILD_DEPENDS=(llvm)
DEPENDS="gcc cargo"
PREPARE_COPY=0

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    config="$(realpath ../config.toml)"
    source="$(realpath ../source)"
    unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP
    python3 "$source/x.py" install --config "$config" --jobs $(nproc) --incremental
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    rsync -av --delete "install/" "$1/"
    # Cannot use STRIP because it is unset in recipe_build
    "${HOST}-strip" -v "$1/bin/"{rustc,rustdoc}
    skip=1
}
