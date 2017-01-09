GIT=https://github.com/redox-os/orbutils.git
BINDIR=/ui/bin

function recipe_stage {
    cp -Rv ui "$1/ui"
}
