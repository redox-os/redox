GIT=https://gitlab.redox-os.org/redox-os/cpal.git
GIT_UPSTREAM=https://github.com/tomaka/cpal.git
BRANCH=redox
CARGOFLAGS="--example beep"

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/beep" "$dest/bin/cpal"
    skip=1
}
