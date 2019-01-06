GIT=https://gitlab.redox-os.org/redox-os/glutin.git
GIT_UPSTREAM=https://github.com/tomaka/glutin.git
BUILD_DEPENDS=(llvm mesa zlib)
BRANCH=redox
CARGOFLAGS="--example window"

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
    cp -v "target/${TARGET}/release/examples/window" "$dest/bin/glutin"
    skip=1
}
