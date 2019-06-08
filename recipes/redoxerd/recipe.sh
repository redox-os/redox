GIT=https://gitlab.redox-os.org/redox-os/redoxer.git

function recipe_update {
    cd daemon
}

function recipe_build {
    cd daemon
}

function recipe_stage {
    mv daemon/target target
}
