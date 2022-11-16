GIT=https://github.com/pop-os/cosmic-text.git
BRANCH=main

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    cargo build --target "$TARGET" --release --package editor-orbclient --features vi
    set +x
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/editor-orbclient" "$dest/bin/cosmic-text"
    skip=1
}
