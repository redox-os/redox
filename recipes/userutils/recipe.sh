GIT=https://github.com/redox-os/userutils.git

function recipe_stage {
    cp -Rv res "$1/etc"
    mkdir -p "$1/bin"
    ln -s id "$1/bin/whoami"
}
