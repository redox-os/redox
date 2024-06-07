GIT=https://gitlab.redox-os.org/redox-os/webrender.git
GIT_UPSTREAM=https://github.com/servo/webrender.git
BRANCH=redox
BUILD_DEPENDS=(freetype2 libpng llvm18 mesa zlib)

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    for rs in $(find examples/ -maxdepth 1 -type f -name '*.rs')
    do
        bin="$(basename "$rs" .rs)"
        set -x
        cargo rustc --target "$TARGET" --release --manifest-path examples/Cargo.toml --bin "$bin" \
            -- \
            -L "${sysroot}/lib" \
            -l static=freetype \
            -l static=png \
            -C link-args="-Wl,-Bstatic $("${PKG_CONFIG}" --libs osmesa) -lz -lstdc++ -lc -lgcc"
        set +x
    done
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    for rs in $(find examples/ -maxdepth 1 -type f -name '*.rs')
    do
        bin="$(basename "$rs" .rs)"
        "${STRIP}" -v "target/$TARGET/release/$bin" -o "$dest/bin/webrender_$bin"
    done
    skip=1
}
