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
        -Wno-dev
        -DCMAKE_CROSSCOMPILING=True
        -DCROSS_TOOLCHAIN_FLAGS_NATIVE="-DCMAKE_TOOLCHAIN_FILE=$native"
        -DCMAKE_INSTALL_PREFIX="/"
        -DLLVM_DEFAULT_TARGET_TRIPLE="$HOST"
        -DLLVM_TARGET_ARCH="$ARCH"
        -DLLVM_TARGETS_TO_BUILD=X86
        -DCMAKE_SYSTEM_NAME=Generic
        -DPYTHON_EXECUTABLE="/usr/bin/python2"
        #-DLLVM_TABLEGEN="/usr/bin/llvm-tblgen-8"
        -DUNIX=1
        -DLLVM_ENABLE_THREADS=Off
        -DLLVM_BUILD_BENCHMARKS=Off
        -DLLVM_INCLUDE_BENCHMARKS=Off
        -DLLVM_BUILD_EXAMPLES=Off
        -DLLVM_INCLUDE_EXAMPLES=Off
        -DLLVM_BUILD_TESTS=Off
        -DLLVM_INCLUDE_TESTS=Off
        -DLLVM_BUILD_UTILS=Off
        -DLLVM_INCLUDE_UTILS=Off
        -target="$HOST"
        -I"$sysroot/include"
        -DCMAKE_CXX_FLAGS='--std=gnu++11 -Wl,--whole-archive -lpthread -Wl,--no-whole-archive'
        -DLLVM_TOOL_LTO_BUILD=Off
        -DLLVM_TOOL_LLVM_PROFDATA_BUILD=Off
        -DLLVM_TOOL_LLI_BUILD=Off
        -DLLVM_TOOL_RDOBJ_BUILD=Off
        -DLLVM_TOOL_LLVM_COV_BUILD=Off
        -DLLVM_TOOL_LLVM_XRAY_BUILD=Off
        -DLLVM_TOOL_LLVM_LTO2_BUILD=Off
        -DLLVM_TOOL_LLVM_LTO_BUILD=Off
        -DLLVM_TOOL_LLVM_RTDYLD_BUILD=Off
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
