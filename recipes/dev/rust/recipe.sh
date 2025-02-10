GIT=https://gitlab.redox-os.org/redox-os/rust.git
BRANCH=redox-2024-05-11
BUILD_DEPENDS=(llvm18-shared zlib libgcc)
DEPENDS="gcc13 cargo"
PREPARE_COPY=0

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_build {
    unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP
    export MAGIC_EXTRA_RUSTFLAGS="-C link-args=-lz"
    python3 "${COOKBOOK_SOURCE}/x.py" install --config "${COOKBOOK_RECIPE}/config.toml" --jobs $(nproc) --incremental
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    rsync -av --delete "install/" "$1/"
    # Cannot use STRIP because it is unset in recipe_build
    #TODO: rustdoc
    "${HOST}-strip" -v "$1/bin/rustc"
    skip=1
}
