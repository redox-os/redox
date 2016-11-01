GIT=https://github.com/jackpot51/pixelcannon.git

function recipe_stage {
    mkdir -pv "$1/apps/pixelcannon"
    cp -Rv assets "$1/apps/pixelcannon"
}
