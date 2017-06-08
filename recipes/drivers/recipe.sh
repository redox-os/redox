GIT=https://github.com/redox-os/drivers.git
CARGOFLAGS=--all

function recipe_version {
    echo "0.1.1"
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/etc"
    cp -v pcid.toml "$1/etc/pcid.toml"
}
