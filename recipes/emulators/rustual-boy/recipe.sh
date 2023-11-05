GIT=https://gitlab.redox-os.org/redox-os/rustual-boy.git
GIT_UPSTREAM=https://github.com/emu-rs/rustual-boy.git
BRANCH="redox"
DEPENDS="orbital"

function recipe_build {
    cd rustual-boy-cli
}

function recipe_stage {
    mv rustual-boy-cli/target target
}
