GIT=https://gitlab.redox-os.org/redox-os/cargo.git
GIT_UPSTREAM=https://github.com/rust-lang/cargo.git
BRANCH=redox
BUILD_DEPENDS=(curl openssl zlib)

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export DEP_OPENSSL_ROOT="$sysroot"
    export OPENSSL_DIR="$sysroot"
    export DEP_Z_ROOT="$sysroot"
}
