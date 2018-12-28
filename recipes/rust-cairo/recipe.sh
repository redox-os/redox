GIT=https://gitlab.redox-os.org/redox-os/rust-cairo.git
BUILD_DEPENDS=(cairo zlib pixman freetype libpng)
CARGOFLAGS="--example gui"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    cp -p "$ROOT/Xargo.toml" "Xargo.toml"
    xargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -l cairo \
	-l pixman-1 \
	-l freetype \
	-l png \
	-l z 
    skip=1
}

