GIT=https://gitlab.redox-os.org/redox-os/kernel.git
BUILD_DEPENDS=(drivers init nulld ramfs randd redoxfs zerod)

function recipe_build {
    export INITFS_FOLDER="$(realpath ../sysroot)"
    mkdir -pv "$INITFS_FOLDER/etc"
    cp -v "$(realpath ../init.rc)" "$INITFS_FOLDER/etc/init.rc"
    cargo rustc \
        --lib \
        --target "${ARCH}-unknown-kernel" \
        --release \
		-Z build-std=core,alloc \
        -- \
        -C soft-float \
        -C debuginfo=2 \
		-C lto \
        --emit link=libkernel.a
    ../kernel_ld.sh "${LD}" \
        --gc-sections \
        -z max-page-size=0x1000 \
        -T "linkers/${ARCH}.ld" \
        -o kernel \
        libkernel.a
    "${OBJCOPY}" \
        --only-keep-debug \
        kernel \
        kernel.sym
    "${OBJCOPY}" \
        --strip-debug \
        kernel
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
    cp -v kernel "$dest"
    skip=1
}
