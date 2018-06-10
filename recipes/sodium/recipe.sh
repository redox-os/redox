GIT=https://gitlab.redox-os.org/redox-os/sodium.git
BINDIR="/ui/bin"
CARGOFLAGS="--features orbital"

function recipe_stage {
    mkdir -pv "$1/ui/apps"
    cp -v manifest "$1/ui/apps/sodium"
    mkdir -pv "$1/ui/icons"
    cp -v icon.png "$1/ui/icons/sodium.png"
}
