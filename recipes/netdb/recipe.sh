GIT=https://gitlab.redox-os.org/redox-os/netdb.git

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
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
    cp -r  * ../stage/
    skip=1
}
