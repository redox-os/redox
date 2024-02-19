GIT=https://gitlab.redox-os.org/redox-os/periodictable.git
BINDIR="/usr/bin"
DEPENDS="orbital"

function recipe_stage {
    mkdir -pv "$1/ui/apps"
    cp -v pkg/manifest "$1/ui/apps/periodictable"
    mkdir -pv "$1/ui/icons"
    cp -v pkg/icon.png "$1/ui/icons/periodictable.png"
}
