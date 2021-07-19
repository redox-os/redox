GIT=https://gitlab.redox-os.org/redox-os/glium.git
GIT_UPSTREAM=https://github.com/glium/glium.git
BUILD_DEPENDS=(llvm mesa zlib)
BRANCH=redox
CARGOFLAGS="--example teapot"

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    cargo rustc --target "$TARGET" --release ${CARGOFLAGS} \
        -- \
        -L "${sysroot}/lib" \
        -C link-args="-Wl,-Bstatic $("${PKG_CONFIG}" --libs osmesa) -lz -lstdc++ -lc -lgcc"
    set +x
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/teapot" "$dest/bin/glium"
    skip=1
}
