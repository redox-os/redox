GIT=https://gitlab.redox-os.org/redox-os/llvm.git
GIT_UPSTREAM=https://github.com/rust-lang/llvm.git
BRANCH=redox

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
    source="$(realpath ../source)"
    sysroot="$(realpath ../sysroot)"
    CMAKE_ARGS=(
        -DCMAKE_AR="$(which "${AR}")"
        -DCMAKE_BUILD_TYPE=Release
        -DCMAKE_CROSSCOMPILING=True
        -DCMAKE_CXX_FLAGS="--std=gnu++11 -Wl,--whole-archive -lpthread -Wl,--no-whole-archive"
        -DCMAKE_RANLIB="$(which "${RANLIB}")"
        -DCMAKE_INSTALL_PREFIX="/"
        -DCMAKE_SYSTEM_NAME=Generic
        -DCROSS_TOOLCHAIN_FLAGS_NATIVE="-DCMAKE_TOOLCHAIN_FILE=$native"
        -DLLVM_BUILD_BENCHMARKS=Off
        -DLLVM_BUILD_EXAMPLES=Off
        -DLLVM_BUILD_TESTS=Off
        -DLLVM_BUILD_UTILS=Off
        -DLLVM_DEFAULT_TARGET_TRIPLE="$HOST"
        -DLLVM_ENABLE_LTO=On
        -DLLVM_ENABLE_THREADS=On
        -DLLVM_INCLUDE_BENCHMARKS=Off
        -DLLVM_INCLUDE_EXAMPLES=Off
        -DLLVM_INCLUDE_TESTS=Off
        -DLLVM_INCLUDE_UTILS=Off
        -DLLVM_OPTIMIZED_TABLEGEN=On
        #-DLLVM_TABLEGEN="/usr/bin/llvm-tblgen-8"
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
        -DPYTHON_EXECUTABLE="/usr/bin/python2"
        -DUNIX=1
        -target="$HOST"
        -I"$sysroot/include"
        -Wno-dev
    )
    cmake "${CMAKE_ARGS[@]}" "$source"
    make -j$(nproc)
    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    make DESTDIR="$dest" install
    find "$dest"/bin -exec $STRIP {} ';' 2> /dev/null
    skip=1
}
