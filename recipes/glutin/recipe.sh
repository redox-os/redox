GIT=https://gitlab.redox-os.org/redox-os/glutin.git
GIT_UPSTREAM=https://github.com/tomaka/glutin.git
BUILD_DEPENDS=(mesa)
BRANCH=redox
CARGOFLAGS="--example window"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    cp -p "$ROOT/Xargo.toml" "Xargo.toml"
    xargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -l OSMesa \
        -l glapi \
        -l stdc++ \
        -C link-args="-Wl,--whole-archive -lpthread -Wl,--no-whole-archive"
    skip=1
}
