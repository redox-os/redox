GIT=https://github.com/redox-os/userutils.git

function recipe_stage {
    cp -Rv res "$1/etc"
}
