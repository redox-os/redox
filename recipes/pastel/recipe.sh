GIT=https://github.com/robbycerantola/pastel.git


function recipe_stage {
    mkdir "$1/ui"
    cp -rv res "$1/ui/pastel"
}
