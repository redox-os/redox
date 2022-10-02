GIT=https://gitlab.redox-os.org/redox-os/glutin.git
GIT_UPSTREAM=https://github.com/rust-windowing/glutin.git
BUILD_DEPENDS=(llvm mesa zlib)
BRANCH=redox-0.29

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    cargo rustc --target "$TARGET" --release --package glutin_examples --example window \
        -- \
        -L "${sysroot}/lib" \
        -C link-args="-Wl,-Bstatic $("${PKG_CONFIG}" --libs osmesa) -lz -lstdc++ -lc -lgcc"
    set +x
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/window" "$dest/bin/glutin"
    skip=1
}
