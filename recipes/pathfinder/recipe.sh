GIT=https://gitlab.redox-os.org/redox-os/pathfinder.git
BRANCH=redox
GIT_UPSTREAM=https://github.com/servo/pathfinder.git
BUILD_DEPENDS=(llvm mesa zlib)
BRANCH=redox
CARGOFLAGS="--manifest-path examples/canvas_glutin_minimal/Cargo.toml"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    cp -p "$ROOT/Xargo.toml" "Xargo.toml"
    set -x
    xargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -C link-args="$("${PKG_CONFIG}" --libs osmesa) -lglapi -lz -lstdc++ -lc -lgcc"
    set +x
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/canvas_glutin_minimal" "$dest/bin/pathfinder"
    skip=1
}
