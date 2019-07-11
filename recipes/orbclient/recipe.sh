
GIT=https://gitlab.redox-os.org/redox-os/orbclient.git
CARGOFLAGS="--example simple"

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/bin"
    cp -v "target/${TARGET}/release/examples/simple" "$dest/bin/orbclient"
    skip=1
}
