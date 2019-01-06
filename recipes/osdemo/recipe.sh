BUILD_DEPENDS=(liborbital llvm mesa mesa_glu zlib)

function recipe_version {
    printf "1.0.0"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

function recipe_prepare {
    rm -rf source
    mkdir source
    cp osdemo.c source
}

function recipe_build {
    sysroot="$(realpath ../sysroot)"
    set -x
    "${CXX}" -O2 -I "$sysroot/include" -L "$sysroot/lib" osdemo.c -o osdemo -lorbital $("${PKG_CONFIG}" --libs glu) -lglapi -lz
    set +x
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
    mkdir -pv "$dest/bin"
    cp -v "osdemo" "$dest/bin/osdemo"
    skip=1
}
