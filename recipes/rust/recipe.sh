GIT=https://github.com/ids1024/rust.git
BRANCH=compile-redox

LLVM_PREFIX=$PWD/build/llvm-root
SYSROOT=/usr/$HOST
unset AR AS CC CXX LD NM OBJCOPY OBJDUMP RANLIB READELF STRIP


LLVM_CMAKE_ARGS=(-Wno-dev -DCMAKE_CROSSCOMPILING=True -DCMAKE_INSTALL_PREFIX="$LLVM_PREFIX" -DLLVM_DEFAULT_TARGET_TRIPLE=$HOST -DLLVM_TARGET_ARCH=$ARCH -DLLVM_TARGETS_TO_BUILD=X86 -DCMAKE_SYSTEM_NAME=Generic -DPYTHON_EXECUTABLE=/usr/bin/python2 -DUNIX=1 -DLLVM_ENABLE_THREADS=Off -DLLVM_INCLUDE_TESTS=OFF -target=$HOST -DLLVM_TABLEGEN=/usr/bin/llvm-tblgen -I"$SYSROOT/include" -DCMAKE_CXX_FLAGS='--std=gnu++11' -DLLVM_TOOL_LTO_BUILD=Off -DLLVM_TOOL_LLVM_PROFDATA_BUILD=Off -DLLVM_TOOL_LLI_BUILD=Off -DLLVM_TOOL_RDOBJ_BUILD=Off -DLLVM_TOOL_LLVM_COV_BUILD=Off -DLLVM_TOOL_LLVM_XRAY_BUILD=Off -DLLVM_TOOL_LLVM_LTO2_BUILD=Off -DLLVM_TOOL_LLVM_LTO_BUILD=Off -DLLVM_TOOL_LLVM_RTDYLD_BUILD=Off)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    # Download patched LLVM
    if [ -d llvm-redox ]
    then
        git -C llvm-redox pull
    else
        git clone https://github.com/ids1024/llvm.git -b redox2 --depth 1 llvm-redox
    fi

    # Build LLVM
    rm -rf $LLVM_PREFIX
    mkdir $LLVM_PREFIX
    mkdir -p llvm-redox/build
    pushd llvm-redox/build
        CC=$HOST-gcc CXX=$HOST-g++ cmake "${LLVM_CMAKE_ARGS[@]}" ..
	make -j"$(nproc)"
	make install
    popd

    cp ../{config.toml,llvm-config} ./
    python x.py build
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
    libpath="$1/lib/rustlib/${RUST_HOST}/lib"
    mkdir -p "$binpath" "$libpath"
    cp -fv "build/${RUST_HOST}/stage2/bin/rustc" "$binpath"
    ${HOST}-strip "$binpath/rustc"
    cp -fv $(find build/${RUST_HOST}/stage2/lib/rustlib/${RUST_HOST}/lib/ -type f | grep -v librustc) "$libpath"
    skip=1
}
