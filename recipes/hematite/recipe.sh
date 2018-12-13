GIT=https://gitlab.redox-os.org/redox-os/hematite.git
GIT_UPSTREAM=https://github.com/PistonDevelopers/hematite.git
BUILD_DEPENDS=(mesa)
BRANCH=redox

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
