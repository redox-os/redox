GIT=https://gitlab.redox-os.org/redox-os/bootloader.git

function recipe_version {
    echo "0.1.0"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    nasm -f bin -o bootloader -D "ARCH_${ARCH}" -i"${ARCH}/" "${ARCH}/disk.asm"
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
    cp -v bootloader "$dest"
    skip=1
}
