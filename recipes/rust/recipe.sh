GIT=https://gitlab.redox-os.org/redox-os/rust.git
BRANCH=redox-2023-09-07
BUILD_DEPENDS=(llvm)
DEPENDS="gcc cargo"
PREPARE_COPY=0

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_build {
    unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP
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
