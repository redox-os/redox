GIT=https://github.com/redox-os/uutils.git
CARGOFLAGS="--no-default-features --features redox"

function recipe_stage {
    mkdir -p "$1/bin"
    ln -s uutils "$1/bin/chmod"
    ln -s uutils "$1/bin/env"
    ln -s uutils "$1/bin/expr"
    ln -s uutils "$1/bin/install"
    ln -s uutils "$1/bin/ls"
    ln -s uutils "$1/bin/mktemp"
}
