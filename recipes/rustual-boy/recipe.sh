GIT=https://github.com/xtibor/rustual-boy.git
BRANCH=redox

function recipe_update {
    cd rustual-boy-cli
}

function recipe_build {
    cd rustual-boy-cli
}

function recipe_stage {
    mv rustual-boy-cli/target target
}
