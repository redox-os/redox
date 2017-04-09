GIT=https://github.com/redox-os/drivers
CARGOFLAGS=--all
BINDIR="/sbin"

function recipe_update {
    cp ../Cargo.toml ./
}
