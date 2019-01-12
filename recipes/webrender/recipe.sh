GIT=https://gitlab.redox-os.org/redox-os/webrender.git
GIT_UPSTREAM=https://github.com/servo/webrender.git
BRANCH=redox
BUILD_DEPENDS=(freetype libpng llvm mesa zlib)
CARGOFLAGS="--manifest-path examples/Cargo.toml --bin basic"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    cp -p "$ROOT/Xargo.toml" "Xargo.toml"
    set -x
    xargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -l static=freetype \
        -l static=png \
        -C link-args="$("${PKG_CONFIG}" --libs osmesa) -lglapi -lz -lstdc++ -lc -lgcc"
    set +x
    skip=1
}


function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/basic" "$dest/bin/webrender"
    skip=1
}
