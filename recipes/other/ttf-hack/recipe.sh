VERSION=3.003
TAR="https://github.com/source-foundry/Hack/releases/download/v$VERSION/Hack-v$VERSION-ttf.tar.xz"

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
    dest="$(realpath "$1")"
    for file in *.ttf; do
        install -D -m 644 "$file" "$dest/ui/fonts/Mono/Hack/$file"
    done
    skip=1
}
