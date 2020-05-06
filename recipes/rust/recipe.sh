GIT=https://gitlab.redox-os.org/redox-os/rust.git
BRANCH=redox-2019-11-25
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
    python "$source/x.py" dist --config "$config" --jobs $(nproc) --incremental
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
    binpath="$1/bin"
    libpath="$1/lib"
    cp -frv "build/${TARGET}/stage2/bin" "$binpath"
    cp -frv "build/${TARGET}/stage2/lib" "$libpath"
    ${STRIP} "$binpath/"*
    skip=1
}
