GIT=https://github.com/pciutils/pciids.git

function recipe_build {
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"

    install -d "${dest}/share/misc/"
    install pci.ids "${dest}/share/misc/"

    skip=1
}
