GIT=https://gitlab.redox-os.org/redox-os/rust-cairo-demo.git
BUILD_DEPENDS=(cairo expat fontconfig freetype libpng pixman zlib)

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

