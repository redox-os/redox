GIT=https://github.com/redox-os/drivers.git
CARGOFLAGS=--all

function recipe_version {
    echo "0.1.0"
    return 1
}

function recipe_update {
    cp ../Cargo.toml ./
}
