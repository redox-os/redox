GIT=https://github.com/jackpot51/pixelcannon.git

function recipe_stage {
    mkdir -pv "$1/apps/pixelcannon"
    cp -Rv assets "$1/apps/pixelcannon"
    mkdir -pv "$1/ui/apps"
    cp -v manifest "$1/ui/apps/pixelcannon"
    mkdir -pv "$1/ui/bin"
    cp -v "target/$TARGET/release/pixelcannon" "$1/ui/bin"
}
