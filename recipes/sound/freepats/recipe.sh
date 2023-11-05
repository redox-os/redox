VERSION=20060219
GIT=https://gitlab.redox-os.org/redox-os/freepats.git

function recipe_version {
    echo "$VERSION"
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
    mkdir -pv "$1/share/freepats"
    cp -Rv ./* "$1/share/freepats"
    mkdir -pv "$1/etc/timidity"
    echo "dir /share/freepats" > "$1/etc/timidity/freepats.cfg"
    echo "source /share/freepats/freepats.cfg" >> "$1/etc/timidity/freepats.cfg"
    skip=1
}
