GIT=https://github.com/redox-os/cargo.git
BRANCH=redox_rebase
BUILD_DEPENDS=(openssl)

function recipe_build {
    export OPENSSL_DIR="$PWD/../sysroot"
}
