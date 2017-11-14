GIT=https://github.com/redox-os/rust.git
BRANCH=compile-redox-stage-0
DEPENDS="gcc cargo"

LLVM_GIT="https://github.com/redox-os/llvm.git"
LLVM_SOURCE="$(realpath llvm-source)"
LLVM_BUILD="$(realpath llvm-build)"
LLVM_PREFIX="$(realpath llvm-prefix)"
SYSROOT="/usr/$HOST"
unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP


LLVM_CMAKE_ARGS=(-Wno-dev -DCMAKE_CROSSCOMPILING=True -DCMAKE_INSTALL_PREFIX="$LLVM_PREFIX" -DLLVM_DEFAULT_TARGET_TRIPLE=$HOST -DLLVM_TARGET_ARCH=$ARCH -DLLVM_TARGETS_TO_BUILD=X86 -DCMAKE_SYSTEM_NAME=Generic -DPYTHON_EXECUTABLE=/usr/bin/python2 -DUNIX=1 -DLLVM_ENABLE_THREADS=Off -DLLVM_INCLUDE_TESTS=OFF -target=$HOST -DLLVM_TABLEGEN=/usr/bin/llvm-tblgen -I"$SYSROOT/include" -DCMAKE_CXX_FLAGS='--std=gnu++11' -DLLVM_TOOL_LTO_BUILD=Off -DLLVM_TOOL_LLVM_PROFDATA_BUILD=Off -DLLVM_TOOL_LLI_BUILD=Off -DLLVM_TOOL_RDOBJ_BUILD=Off -DLLVM_TOOL_LLVM_COV_BUILD=Off -DLLVM_TOOL_LLVM_XRAY_BUILD=Off -DLLVM_TOOL_LLVM_LTO2_BUILD=Off -DLLVM_TOOL_LLVM_LTO_BUILD=Off -DLLVM_TOOL_LLVM_RTDYLD_BUILD=Off)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_fetch {
    if [ ! -d "$LLVM_SOURCE" ]
    then
        git clone "$LLVM_GIT" -b redox --depth 1 "$LLVM_SOURCE"
    fi

    pushd "$LLVM_SOURCE" > /dev/null
    git remote set-url origin "$LLVM_GIT"
    git fetch origin
    git pull
    git submodule sync --recursive
    git submodule update --init --recursive
    popd > /dev/null
}

function recipe_prepare {
    rm -rf "$LLVM_PREFIX"
    mkdir -p "$LLVM_PREFIX"

    rm -rf "$LLVM_BUILD"
    mkdir "$LLVM_BUILD"
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    # Build LLVM
    pushd "$LLVM_BUILD"
        CC=$HOST-gcc CXX=$HOST-g++ cmake "${LLVM_CMAKE_ARGS[@]}" "${LLVM_SOURCE}"
        make -j$(nproc)
        make install
    popd

    python x.py build --config ../config.toml --jobs $(nproc)
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
    binpath="$1/bin"
    libpath="$1/lib/rustlib/${TARGET}/lib"
    mkdir -p "$binpath" "$libpath"
    cp -fv "build/${TARGET}/stage2/bin/rustc" "$binpath"
    ${HOST}-strip "$binpath/rustc"
    cp -fv $(find build/${TARGET}/stage2/lib/rustlib/${TARGET}/lib/ -type f | grep -v librustc) "$libpath"
    skip=1
}
