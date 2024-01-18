GIT=https://github.com/jackpot51/pixelcannon.git
BINDIR=/usr/bin
DEPENDS="orbital"

function recipe_stage {
    mkdir -pv "$1/apps/pixelcannon"
    cp -Rv assets "$1/apps/pixelcannon"
    mkdir -pv "$1/ui/apps"
    cp -v manifest "$1/ui/apps/pixelcannon"
}
