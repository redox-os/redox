GIT=https://github.com/redox-os/rustual-boy.git
GIT_UPSTREAM=https://github.com/emu-rs/rustual-boy.git

function recipe_update {
    cd rustual-boy-cli
}

function recipe_build {
    cd rustual-boy-cli
}

function recipe_stage {
    mv rustual-boy-cli/target target
}
