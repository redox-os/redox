VERSION=1.471
GIT=https://gitlab.redox-os.org/redox-os/generaluser-gs.git

function recipe_version {
    echo "$VERSION"
    skip=1
}

function recipe_build {
    echo "skipping build"
    skip=1
}

function recipe_clean {
    echo "skipping clean"
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/share/generaluser-gs"
    cp -Rv ./* "$1/share/generaluser-gs"
    skip=1
}
