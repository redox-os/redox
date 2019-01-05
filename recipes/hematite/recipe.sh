GIT=https://gitlab.redox-os.org/redox-os/hematite.git
GIT_UPSTREAM=https://github.com/PistonDevelopers/hematite.git
BUILD_DEPENDS=(mesa llvm zlib)
BRANCH=redox

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
