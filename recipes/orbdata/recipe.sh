GIT=https://github.com/redox-os/orbdata.git

function recipe_version {
    echo "0.0.1"
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
    mkdir -pv "$1/ui"
    cp -Rv ./* "$1/ui"
    skip=1
}
