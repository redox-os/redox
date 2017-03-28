GIT=https://github.com/redox-os/orbdata.git

function recipe_version {
    echo "0.0.1"
    return 1
}

function recipe_update {
    echo "skipping update"
    return 1
}

function recipe_build {
    echo "skipping build"
    return 1
}

function recipe_test {
    echo "skipping test"
    return 1
}

function recipe_clean {
    echo "skipping clean"
    return 1
}

function recipe_stage {
    mkdir -pv "$1/ui"
    cp -Rv ./* "$1/ui"
    return 1
}
