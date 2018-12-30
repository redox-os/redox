GIT=https://gitlab.redox-os.org/redox-os/winit.git
GIT_UPSTREAM=https://github.com/tomaka/winit.git
BRANCH=redox
CARGOFLAGS="--example window"

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/window" "$dest/bin/winit"
    skip=1
}
