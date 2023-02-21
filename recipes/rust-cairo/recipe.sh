GIT=https://gitlab.redox-os.org/redox-os/rust-cairo.git
BUILD_DEPENDS=(cairo expat fontconfig freetype libpng pixman zlib)
CARGOFLAGS="--example gui"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    cargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -l cairo \
        -l fontconfig \
        -l expat \
    	-l pixman-1 \
    	-l freetype \
    	-l png \
    	-l z
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/gui" "$dest/bin/rust-cairo"
    skip=1
}
