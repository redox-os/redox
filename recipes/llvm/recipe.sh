GIT=https://gitlab.redox-os.org/redox-os/llvm-project.git
GIT_UPSTREAM=https://github.com/rust-lang/llvm-project.git
BRANCH=redox-2022-03-18

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_prepare {
    mkdir -p build
    skip=1
}

function recipe_build {
    native="$(realpath ../native.cmake)"
    source="$(realpath ../source/llvm)"
    sysroot="$(realpath ../sysroot)"
    CMAKE_ARGS=(
        -DCMAKE_AR="$(which "${AR}")"
        -DCMAKE_BUILD_TYPE=Release
        -DCMAKE_CROSSCOMPILING=True
        -DCMAKE_CXX_FLAGS="--std=gnu++11"
        -DCMAKE_EXE_LINKER_FLAGS="-static"
        -DCMAKE_RANLIB="$(which "${RANLIB}")"
        -DCMAKE_INSTALL_PREFIX="/"
        -DCMAKE_INSTALL_INCLUDEDIR="include"
        -DCMAKE_INSTALL_OLDINCLUDEDIR="/include"
        -DCMAKE_SYSTEM_NAME=Generic
        -DCROSS_TOOLCHAIN_FLAGS_NATIVE="-DCMAKE_TOOLCHAIN_FILE=$native"
        -DLLVM_BUILD_BENCHMARKS=Off
        -DLLVM_BUILD_EXAMPLES=Off
        -DLLVM_BUILD_TESTS=Off
        -DLLVM_BUILD_UTILS=Off
        -DLLVM_DEFAULT_TARGET_TRIPLE="$HOST"
        -DLLVM_ENABLE_LTO=Off
        -DLLVM_ENABLE_RTTI=On
        -DLLVM_ENABLE_THREADS=On
        -DLLVM_INCLUDE_BENCHMARKS=Off
        -DLLVM_INCLUDE_EXAMPLES=Off
        -DLLVM_INCLUDE_TESTS=Off
        -DLLVM_INCLUDE_UTILS=Off
        -DLLVM_OPTIMIZED_TABLEGEN=On
        -DLLVM_TARGET_ARCH="$ARCH"
        -DLLVM_TARGETS_TO_BUILD=X86
        -DLLVM_TOOL_LLVM_COV_BUILD=Off
        -DLLVM_TOOL_LLVM_LTO_BUILD=Off
        -DLLVM_TOOL_LLVM_LTO2_BUILD=Off
        -DLLVM_TOOL_LLVM_PROFDATA_BUILD=Off
        -DLLVM_TOOL_LLVM_RTDYLD_BUILD=Off
        -DLLVM_TOOL_LLVM_XRAY_BUILD=Off
        -DLLVM_TOOL_LLI_BUILD=Off
        -DLLVM_TOOL_LTO_BUILD=Off
        -DLLVM_TOOLS_INSTALL_DIR=bin
        -DLLVM_UTILS_INSTALL_DIR=bin
        -DPYTHON_EXECUTABLE="/usr/bin/python2"
        -DUNIX=1
        -Wno-dev
    )
    set -x
    cmake "${CMAKE_ARGS[@]}" "$source"
    "$REDOX_MAKE" -j$(nproc)
    set +x
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    "$REDOX_MAKE" clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    set -x
    "$REDOX_MAKE" DESTDIR="$dest" install
    mv -vT "$dest"/usr/include "$dest/include"
    mv -vT "$dest"/usr/share "$dest/share"
    rmdir -v "$dest"/usr
    find "$dest"/bin -exec "$STRIP" -v {} ';' 2> /dev/null
    set +x
    skip=1
}
