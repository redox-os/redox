GIT=https://github.com/ids1024/cargo.git
BRANCH=redox
BUILD_DEPENDS=(openssl)

function recipe_build {
    export OPENSSL_DIR="$PWD/../sysroot"
}
