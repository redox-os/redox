GIT=https://github.com/uutils/coreutils.git
CARGOFLAGS="--no-default-features --features=generic"

function recipe_build {
    echo "Skipping build of uutils"
    return 1
}
