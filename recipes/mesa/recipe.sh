GIT=https://gitlab.redox-os.org/redox-os/mesa.git
GIT_UPSTREAM=https://gitlab.freedesktop.org/mesa/mesa
BRANCH=redox
BUILD_DEPENDS=(expat zlib)

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    export CFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export CPPFLAGS="-I$sysroot/include -DHAVE_PTHREAD=1"
    export LDFLAGS="-L$sysroot/lib --static"
    #export LLVM_CONFIG="x86_64-unknown-redox-llvm-config"

    # TODO: Fix this annoying shite
    echo "[binaries]" > cross_file.txt
    echo "c = '${CC}'" >> cross_file.txt
    echo "cpp = '${CXX}'" >> cross_file.txt
    echo "ar = '${AR}'" >> cross_file.txt
    echo "strip = '${STRIP}'" >> cross_file.txt
    echo "pkgconfig = '${PKG_CONFIG}'" >> cross_file.txt
    #echo "llvm-config = '${TARGET}-llvm-config'" >> cross_file.txt

    echo "[host_machine]" >> cross_file.txt
    echo "system = 'redox'" >> cross_file.txt
    echo "cpu_family = 'x86_64'" >> cross_file.txt
    echo "cpu = 'x86_64'" >> cross_file.txt
    echo "endian = 'little'" >> cross_file.txt

    echo "[paths]" >> cross_file.txt
    echo "prefix = '/'" >> cross_file.txt
    echo "libdir = 'lib'" >> cross_file.txt
    echo "bindir = 'bin'" >> cross_file.txt

    unset AR
    unset AS
    unset CC
    unset CXX
    unset LD
    unset NM
    unset OBJCOPY
    unset OBJDUMP
    unset PKG_CONFIG
    unset PKG_CONFIG_PATH
    unset RANLIB
    unset READELF
    unset STRIP

    meson . _build \
        --cross-file cross_file.txt \
        --buildtype release \
        --strip \
        -Ddefault_library=static \
        -Dglx=disabled \
        -Dllvm=disabled \
        -Dosmesa=gallium \
        -Dplatforms= \
        -Dshader-cache=disabled \
        -Dshared-llvm=disabled \
        -Dshared-glapi=disabled

    ninja -C _build -v
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
    DESTDIR="$dest" ninja -C _build install
    rm -f "$dest/lib/"*.la
    skip=1
}
