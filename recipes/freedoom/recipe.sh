GIT=https://gitlab.redox-os.org/redox-os/freedoom.git
DEPENDS=(ion prboom)

function recipe_version {
    echo "0.11.3"
    skip=1
}

function recipe_build {
    echo "skipping build"
    skip=1
}

function recipe_clean {
    echo "skipping clean"
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/games" "$1/share/games/doom" "$1/ui/apps" "$1/ui/icons/apps"
    for file in ./*.wad
    do
        game="$(basename "$file" .wad)"

        wad="/share/games/doom/$game.wad"
        cp -v "$file" "$1$wad"

        bin="/games/$game"
        echo "#!/bin/ion" > "$1$bin"
        echo "/games/prboom -geom 800x600 -vidmode 32 -iwad $wad" >> "$1$bin"
        chmod +x "$1$bin"

        echo "name=$game" | sed 's/freedoom/FreeDOOM: Phase /' | sed 's/doom1/DOOM (Shareware)/' > "$1/ui/apps/$game"
        echo "category=Games" >> "$1/ui/apps/$game"
        echo "binary=/games/$game" >> "$1/ui/apps/$game"
        echo "icon=/ui/icons/apps/$game.png" >> "$1/ui/apps/$game"

        cp -v "$game.png" "$1/ui/icons/apps/$game.png"
    done
    skip=1
}
