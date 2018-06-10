GIT=https://gitlab.redox-os.org/redox-os/openttd-opengfx.git

function recipe_version {
    echo "0.5.2"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    echo "skipping build"
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    echo "skipping clean"
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/share/games/openttd/baseset/opengfx"
    cp -Rv ./* "$1/share/games/openttd/baseset/opengfx"
    skip=1
}
