GIT=https://gitlab.redox-os.org/redox-os/rust.git
BRANCH=compile-redox
BUILD_DEPENDS=(llvm)
DEPENDS="gcc cargo"

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP
    python x.py dist --config ../config.toml --jobs $(nproc) --incremental --keep-stage 0
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
    binpath="$1/bin"
    libpath="$1/lib/rustlib/${TARGET}/lib"
    mkdir -p "$binpath" "$libpath"
    cp -fv "build/${TARGET}/stage2/bin/rustc" "$binpath"
    ${STRIP} "$binpath/rustc"
    cp -fv $(find build/${TARGET}/stage2/lib/rustlib/${TARGET}/lib/ -type f | grep -v librustc) "$libpath"
    skip=1
}
