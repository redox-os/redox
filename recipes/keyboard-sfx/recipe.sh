GIT=https://gitlab.redox-os.org/redox-os/keyboard-sfx.git

function recipe_version {
    echo "0.0.1"
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
    mkdir -pv "$1/sfx"
    cp -Rv ./* "$1/sfx"
    skip=1
}
